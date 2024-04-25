use anyhow::Error;
use opentelemetry::{
    global,
    trace::{Span, TraceContextExt, Tracer, TracerProvider},
    Context, KeyValue,
};
use std::env::current_dir;
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;

use fluentci_ext::envhub::Envhub;
use fluentci_ext::Extension;
use fluentci_types::{
    nix::NixArgs,
    process_compose::{self, Process},
    Output,
};

use super::edge::Edge;
use super::vertex::{Runnable, Vertex};

#[derive(Default, Debug, Clone)]
pub struct Volume {
    pub id: String,
    pub label: String,
    pub path: String,
    pub key: String,
}

#[derive(Default, Clone)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub vertices: Vec<Vertex>,
}

impl Into<Process> for Service {
    fn into(self) -> Process {
        Process {
            command: self
                .vertices
                .iter()
                .map(|v| v.command.clone())
                .collect::<Vec<String>>()
                .join(" ; "),
            depends_on: None,
        }
    }
}

pub enum GraphCommand {
    AddVolume(String, String, String),
    AddVertex(
        String,
        String,
        String,
        Vec<String>,
        Arc<Box<dyn Extension + Send + Sync>>,
    ),
    AddEdge(usize, usize),
    Execute(Output),
}

#[derive(Clone)]
pub struct Graph {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub volumes: Vec<Volume>,
    pub services: Vec<Service>,

    tx: Sender<(String, usize)>,
    pub runner: Arc<Box<dyn Extension + Send + Sync>>,
    pub work_dir: String,
    pub nix_args: NixArgs,
}

impl Graph {
    pub fn new(tx: Sender<(String, usize)>, runner: Arc<Box<dyn Extension + Send + Sync>>) -> Self {
        let work_dir = current_dir().unwrap().to_str().unwrap().to_string();
        Graph {
            vertices: Vec::new(),
            volumes: Vec::new(),
            services: Vec::new(),
            edges: Vec::new(),
            tx,
            runner,
            work_dir,
            nix_args: NixArgs::default(),
        }
    }

    pub fn register_service(&mut self, id: &str) {
        // return if vertex at id is not found or is not a service
        if self
            .vertices
            .iter()
            .find(|v| v.id == id && v.label == "asService")
            .is_none()
        {
            return;
        }

        let vertex = self
            .vertices
            .iter()
            .find(|v| v.id == id && v.label == "asService")
            .unwrap();

        // browse the graph dependencies of the service
        let mut visited = vec![false; self.vertices.len()];
        let mut stack = Vec::new();
        let mut service = Service {
            id: id.to_string(),
            name: vertex.command.clone(),
            vertices: Vec::new(),
        };

        let skip = vec![
            "git",
            "git-checkout",
            "git-last-commit",
            "tree",
            "http",
            "cache",
            "file",
            "directory",
            "chmod",
            "withFile",
        ];

        for (i, vertex) in self.vertices.iter().enumerate() {
            if vertex.id == id {
                stack.push(i);
                break;
            }
        }

        while let Some(i) = stack.pop() {
            println!("{} {:?}", i, self.edges);
            if visited[i] {
                continue;
            }
            visited[i] = true;

            if skip.contains(&self.vertices[i].label.as_str()) {
                continue;
            }

            for edge in self.edges.iter().filter(|e| e.to == i) {
                stack.push(edge.from);
            }

            let vertex = self.vertices[i].clone();
            println!("Vertex: {} {} {}", vertex.command, vertex.id, vertex.label);
            service.vertices.push(vertex);
        }

        service.vertices.reverse();
        self.services.push(service);
    }

    pub fn execute(&mut self, command: GraphCommand) {
        match command {
            GraphCommand::AddVolume(id, label, path) => {
                self.volumes.push(Volume {
                    id,
                    label,
                    key: path.clone(),
                    path,
                });
            }
            GraphCommand::AddVertex(id, label, command, needs, runner) => {
                if let Some(vertex) = self.vertices.iter_mut().find(|v| v.id == id) {
                    vertex.needs.extend(needs);
                } else {
                    self.vertices.push(Vertex {
                        id,
                        label,
                        command,
                        needs,
                        runner,
                    });
                }
            }
            GraphCommand::AddEdge(from, to) => {
                self.edges.push(Edge { from, to });
            }
            GraphCommand::Execute(Output::Stdout) => {
                self.execute_graph(Output::Stdout);
            }
            GraphCommand::Execute(Output::Stderr) => {
                self.execute_graph(Output::Stderr);
            }
        }
    }

    pub fn execute_services(&mut self, ctx: &Context) -> Result<(), Error> {
        let tracer_provider = global::tracer_provider();
        let tracer = tracer_provider.versioned_tracer(
            "fluentci-core",
            Some(env!("CARGO_PKG_VERSION")),
            Some("https://opentelemetry.io/schemas/1.17.0"),
            None,
        );

        let process_compose_config = process_compose::ConfigFile {
            version: "0.5".to_string(),
            processes: self
                .services
                .iter()
                .map(|s| (s.name.clone(), s.clone().into()))
                .collect(),
            ..Default::default()
        };
        let yaml = serde_yaml::to_string(&process_compose_config)?;
        println!("{}", yaml);
        Ok(())
    }

    pub fn execute_graph(&mut self, output: Output) {
        let tracer_provider = global::tracer_provider();
        let tracer = tracer_provider.versioned_tracer(
            "fluentci-core",
            Some(env!("CARGO_PKG_VERSION")),
            Some("https://opentelemetry.io/schemas/1.17.0"),
            None,
        );

        let skip = vec![
            "git",
            "git-checkout",
            "git-last-commit",
            "tree",
            "http",
            "cache",
            "file",
            "directory",
            "chmod",
            "withFile",
            "asService",
        ];
        let mut visited = vec![false; self.vertices.len()];
        let mut stack = Vec::new();
        for (i, vertex) in self.vertices.iter().enumerate() {
            if vertex.needs.is_empty() {
                stack.push(i);
            }
        }
        let root_span = tracer.start("root");
        let context = Context::current_with_span(root_span);

        match self.execute_services(&context) {
            Ok(_) => {}
            Err(e) => {
                println!("Error: {}", e);
                self.tx.send(("Error".to_string(), 1)).unwrap();
                return;
            }
        }

        while let Some(i) = stack.pop() {
            let label = &self.vertices[i].label.as_str();
            if visited[i] {
                continue;
            }
            visited[i] = true;
            for edge in self.edges.iter().filter(|e| e.from == i) {
                stack.push(edge.to);
            }

            if skip.contains(&label) {
                continue;
            }

            let (tx, rx) = mpsc::channel();
            let mut span = tracer.start_with_context(
                format!("{} {}", label.to_string(), self.vertices[i].command.clone()),
                &context,
            );

            span.set_attribute(KeyValue::new("command", self.vertices[i].command.clone()));

            if self.vertices[i].label == "withWorkdir" {
                if !Path::new(&self.vertices[i].command).exists() {
                    println!("Error: {}", self.vertices[i].id);
                    span.end();
                    self.tx.send((self.vertices[i].command.clone(), 1)).unwrap();
                    break;
                }
                self.work_dir = self.vertices[i].command.clone();
                span.end();
                continue;
            }

            if self.vertices[i].label == "useEnv" {
                match Envhub::default().r#use(&self.vertices[i].command, &self.work_dir) {
                    Ok(_) => {
                        span.end();
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        span.end();
                        self.tx.send((self.vertices[i].command.clone(), 1)).unwrap();
                        break;
                    }
                }
                span.end();
                continue;
            }

            match self.vertices[i].run(tx, output.clone(), stack.len() == 1, &self.work_dir) {
                Ok(status) => {
                    if !status.success() {
                        println!("Error: {}", self.vertices[i].id);
                        span.end();
                        self.tx.send((self.vertices[i].command.clone(), 1)).unwrap();
                        break;
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    span.end();
                    self.tx.send((self.vertices[i].command.clone(), 1)).unwrap();
                    break;
                }
            };

            if stack.len() == 1 {
                if let Ok(command_output) = rx.recv() {
                    match self.tx.send((command_output, 0)) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
            }
            span.end();
        }

        self.post_execute(&context);
    }

    pub fn execute_vertex(&mut self, id: &str) -> Result<String, Error> {
        let tracer_provider = global::tracer_provider();
        let tracer = tracer_provider.versioned_tracer(
            "fluentci-core",
            Some(env!("CARGO_PKG_VERSION")),
            Some("https://opentelemetry.io/schemas/1.17.0"),
            None,
        );

        let mut result = String::from("");
        let mut visited = vec![false; self.vertices.len()];
        let mut stack = Vec::new();
        let mut index = 0;
        for (i, vertex) in self.vertices.iter().enumerate() {
            if vertex.id == id {
                index = i;
                break;
            }
        }
        stack.push(index);
        while let Some(i) = stack.pop() {
            if visited[i] {
                continue;
            }
            visited[i] = true;
            for edge in self.edges.iter().filter(|e| e.from == i) {
                stack.push(edge.to);
            }
            let (tx, rx) = mpsc::channel();
            let label = self.vertices[i].label.clone();
            let command = self.vertices[i].command.clone();
            let mut span = tracer.start(format!("{} {}", label, command.clone()));
            span.set_attribute(KeyValue::new("command", command.clone()));

            if self.vertices[i].label == "withWorkdir" {
                if !Path::new(&self.vertices[i].command).exists() {
                    span.end();
                    return Err(Error::msg(format!("Error: {}", self.vertices[i].id)));
                }
                self.work_dir = self.vertices[i].command.clone();
                span.end();
                continue;
            }

            match self.vertices[i].run(tx, Output::Stdout, true, &self.work_dir) {
                Ok(status) => {
                    if !status.success() {
                        span.end();
                        return Err(Error::msg(format!("Error: {}", self.vertices[i].id)));
                    }
                    result = rx.recv()?;
                    span.end();
                }
                Err(e) => {
                    span.end();
                    return Err(Error::msg(format!("Error: {}", e)));
                }
            };
        }

        Ok(result)
    }

    pub fn post_execute(&mut self, ctx: &Context) {
        let tracer_provider = global::tracer_provider();
        let tracer = tracer_provider.versioned_tracer(
            "fluentci-core",
            Some(env!("CARGO_PKG_VERSION")),
            Some("https://opentelemetry.io/schemas/1.17.0"),
            None,
        );

        let only = vec!["withCache", "withService"];

        let mut visited = vec![false; self.vertices.len()];
        let mut stack = Vec::new();
        for (i, vertex) in self.vertices.iter().enumerate() {
            if vertex.needs.is_empty() {
                stack.push(i);
            }
        }

        while let Some(i) = stack.pop() {
            let label = &self.vertices[i].label.as_str();
            if visited[i] {
                continue;
            }
            visited[i] = true;
            for edge in self.edges.iter().filter(|e| e.from == i) {
                stack.push(edge.to);
            }

            if !only.contains(&label) {
                continue;
            }

            let mut span = tracer.start_with_context(
                format!("{} {}", label.to_string(), self.vertices[i].command.clone()),
                &ctx,
            );
            span.set_attribute(KeyValue::new("command", self.vertices[i].command.clone()));
            let (tx, rx) = mpsc::channel();
            match self.vertices[i].runner.post_setup(tx) {
                Ok(_) => {
                    span.end();
                }
                Err(e) => {
                    println!("Error: {}", e);
                    span.end();
                    break;
                }
            }
            rx.recv().unwrap();
        }
    }

    pub fn size(&self) -> usize {
        self.vertices.len()
    }

    pub fn reset(&mut self) {
        self.vertices.clear();
        self.edges.clear();
        self.work_dir = current_dir().unwrap().to_str().unwrap().to_string();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use fluentci_ext::runner::Runner;

    use super::*;

    #[test]
    fn test_graph() {
        let (tx, _) = mpsc::channel();
        let mut graph = Graph::new(tx, Arc::new(Box::new(Runner::default())));
        graph.execute(GraphCommand::AddVertex(
            "1".into(),
            "A".into(),
            "echo A".into(),
            vec![],
            Arc::new(Box::new(Runner::default())),
        ));
        graph.execute(GraphCommand::AddVertex(
            "2".into(),
            "B".into(),
            "echo B".into(),
            vec!["1".into()],
            Arc::new(Box::new(Runner::default())),
        ));
        graph.execute(GraphCommand::AddVertex(
            "3".into(),
            "C".into(),
            "echo C".into(),
            vec!["1".into()],
            Arc::new(Box::new(Runner::default())),
        ));
        graph.execute(GraphCommand::AddVertex(
            "4".into(),
            "D".into(),
            "echo D".into(),
            vec!["2".into(), "3".into()],
            Arc::new(Box::new(Runner::default())),
        ));
        graph.execute(GraphCommand::AddEdge(0, 1));
        graph.execute(GraphCommand::AddEdge(0, 2));
        graph.execute(GraphCommand::AddEdge(1, 3));
        graph.execute(GraphCommand::AddEdge(2, 3));

        assert_eq!(graph.size(), 4);
        assert_eq!(graph.vertices[0].id, "1");
        assert_eq!(graph.vertices[1].id, "2");
        assert_eq!(graph.vertices[2].id, "3");
        assert_eq!(graph.vertices[3].id, "4");
        assert_eq!(graph.vertices[0].label, "A");
        assert_eq!(graph.vertices[1].label, "B");
        assert_eq!(graph.vertices[2].label, "C");
        assert_eq!(graph.vertices[3].label, "D");
        assert_eq!(graph.vertices[0].command, "echo A");
        assert_eq!(graph.vertices[1].command, "echo B");
        assert_eq!(graph.vertices[2].command, "echo C");
        assert_eq!(graph.vertices[3].command, "echo D");

        graph.execute(GraphCommand::Execute(Output::Stdout));

        graph.reset();

        assert_eq!(graph.size(), 0);
    }
}
