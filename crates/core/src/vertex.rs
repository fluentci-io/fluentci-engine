pub trait Runnable {
    fn run(&self);
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: String,
    pub label: String,
    pub command: String,
    pub needs: Vec<String>,
}

impl Runnable for Vertex {
    fn run(&self) {
        println!("Execute: {}", self.id);
        println!("Command:");
        println!("{}", self.command);
    }
}
