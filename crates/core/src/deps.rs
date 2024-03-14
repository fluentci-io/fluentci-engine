use std::process::ExitStatus;
use std::sync::mpsc::Sender;

use super::edge::Edge;
use super::vertex::{Runnable, Vertex};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Output {
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub enum GraphCommand {
    AddVertex(String, String, String, Vec<String>),
    AddEdge(usize, usize),
    Execute(Output),
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    tx: Sender<(String, ExitStatus)>,
}

impl Graph {
    pub fn new(tx: Sender<(String, ExitStatus)>) -> Self {
        Graph {
            vertices: Vec::new(),
            edges: Vec::new(),
            tx,
        }
    }

    pub fn execute(&mut self, command: GraphCommand) {
        match command {
            GraphCommand::AddVertex(id, label, command, needs) => {
                if let Some(vertex) = self.vertices.iter_mut().find(|v| v.id == id) {
                    vertex.needs.extend(needs);
                } else {
                    self.vertices.push(Vertex {
                        id,
                        label,
                        command,
                        needs,
                    });
                }
            }
            GraphCommand::AddEdge(from, to) => {
                self.edges.push(Edge { from, to });
            }
            GraphCommand::Execute(Output::Stdout) => {
                let mut visited = vec![false; self.vertices.len()];
                let mut stack = Vec::new();
                for (i, vertex) in self.vertices.iter().enumerate() {
                    if vertex.needs.is_empty() {
                        stack.push(i);
                    }
                }
                while let Some(i) = stack.pop() {
                    if visited[i] {
                        continue;
                    }
                    visited[i] = true;
                    for edge in self.edges.iter().filter(|e| e.from == i) {
                        stack.push(edge.to);
                    }

                    let status =
                        self.vertices[i].run(self.tx.clone(), Output::Stdout, stack.len() == 1);
                    if !status.success() {
                        println!("Error: {}", self.vertices[i].id);
                        break;
                    }
                }

                self.reset();
            }
            GraphCommand::Execute(Output::Stderr) => {
                let mut visited = vec![false; self.vertices.len()];
                let mut stack = Vec::new();
                for (i, vertex) in self.vertices.iter().enumerate() {
                    if vertex.needs.is_empty() {
                        stack.push(i);
                    }
                }
                while let Some(i) = stack.pop() {
                    if visited[i] {
                        continue;
                    }
                    visited[i] = true;
                    for edge in self.edges.iter().filter(|e| e.from == i) {
                        stack.push(edge.to);
                    }
                    let status =
                        self.vertices[i].run(self.tx.clone(), Output::Stderr, stack.len() == 1);
                    if !status.success() {
                        println!("Error: {}", self.vertices[i].id);
                        break;
                    }
                }

                self.reset();
            }
        }
    }

    pub fn size(&self) -> usize {
        self.vertices.len()
    }

    pub fn reset(&mut self) {
        self.vertices.clear();
        self.edges.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn test_graph() {
        let (tx, _) = mpsc::channel();
        let mut graph = Graph::new(tx);
        graph.execute(GraphCommand::AddVertex(
            "1".into(),
            "A".into(),
            "echo A".into(),
            vec![],
        ));
        graph.execute(GraphCommand::AddVertex(
            "2".into(),
            "B".into(),
            "echo B".into(),
            vec!["1".into()],
        ));
        graph.execute(GraphCommand::AddVertex(
            "3".into(),
            "C".into(),
            "echo C".into(),
            vec!["1".into()],
        ));
        graph.execute(GraphCommand::AddVertex(
            "4".into(),
            "D".into(),
            "echo D".into(),
            vec!["2".into(), "3".into()],
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

        assert_eq!(graph.size(), 0);
    }
}
