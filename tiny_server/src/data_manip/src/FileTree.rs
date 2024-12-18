use std::sync::Arc;

struct Node{
    name: String,
    children: Vec<Arc<Node>>,
    parent: Option<Arc<Node>>,
    is_file: bool,
    size: usize,
}

impl Node{
    pub fn new(name: String, is_file: bool, size: usize) -> Node{
        Node{
            name,
            children: Vec::new(),
            parent: None,
            is_file,
            size,
        }
    }

    pub fn add_child(&mut self, child: Arc<Node>){
        self.children.push(child);
    }

    pub fn set_parent(&mut self, parent: Option<Arc<Node>>){
        self.parent = parent;
    }
}