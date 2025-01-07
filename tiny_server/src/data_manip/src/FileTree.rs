use std::cell::RefCell;
use std::fs::{self, Metadata};
use std::path::Path;
use std::collections::{HashMap, VecDeque};
use std::sync::Weak;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

enum FileBuildErr{
    IoErr(std::io::Error),
    PermissionErr(String),
    NotFoundErr(String),
    InvalidPath(String),
    IsSymlink(String),
}

type NodeRef = Arc<RefCell<Node>>;
type ParentRef = Option<Weak<RefCell<Node>>>;

struct Node{
    name: String,
    children: HashMap<String, NodeRef>,
    parent: ParentRef,
    is_file: bool,
    size: usize,
    date_created: u64,
    date_modified: u64,
    views_by_week: [u32; 7],
    likes: u32,
    
}

struct FileTree{
    root: NodeRef,
    by_popularity: VecDeque<NodeRef>,
    by_recency: VecDeque<NodeRef>,
    

}

impl Node{

    fn add_child(&mut self, child: NodeRef){
        let borrowed_name = child.borrow().name.clone();
        self.children.insert(borrowed_name, child);
    }

    fn get_child(&self, name: &str) -> Option<NodeRef>{
        self.children.get(name).cloned()
    }

    async fn get_children(&self) -> Vec<NodeRef>{
        self.children.values().map(|x| x.clone()).collect()
    }

    fn root_from_file(path_str: &str, parent: ParentRef) -> Result<NodeRef, FileBuildErr>{
        let path = Path::new(path_str);

        let metadata = match path.metadata(){
            Ok(meta) => meta,
            Err(e) => return match e.kind(){
                std::io::ErrorKind::PermissionDenied => Err(FileBuildErr::PermissionErr(path_str.to_string())),
                std::io::ErrorKind::NotFound => Err(FileBuildErr::NotFoundErr(path_str.to_string())),
                _ => Err(FileBuildErr::IoErr(e)),
            }
        };

        let name = match path.file_name(){
            Some(name) => name.to_string_lossy().to_string(),
            None => return Err(FileBuildErr::InvalidPath(path_str.to_string())),
        };

        let is_file = metadata.is_file();
        let size = metadata.len();
        let date_created = metadata.created().unwrap().elapsed().unwrap().as_secs();
        let mut node = Node{
            name,
            children: HashMap::new(),
            parent: None,
            is_file,
            size: size as usize,
            date_created,
            date_modified: date_created,
            views_by_week: [0; 7],
            likes: 0,
        };
        node.parent = parent;
        let node_ref = Arc::new(RefCell::new(node));
        if metadata.is_symlink(){
            return Err(FileBuildErr::IsSymlink(path_str.to_string()));
        }
        
        if !is_file{
            let entries = match fs::read_dir(path){
                Ok(entries) => entries,
                Err(e) => return Err(FileBuildErr::IoErr(e)),
            };

            for entry in entries{
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => return Err(FileBuildErr::IoErr(e)),
                };

                let child_path = entry.path();
                let child_path_str = child_path.to_str().unwrap_or("");
                if child_path_str.is_empty() {
                    continue;
                }

                let child_node = Node::root_from_file(child_path_str, Some(Arc::downgrade(&node_ref)))?;
                node_ref.borrow_mut().add_child(child_node);

            }
        }

        Ok(node_ref)
        
    }
}

impl FileTree{
    pub async fn get_file(&self, path: &str) -> Option<NodeRef>{
        let return_path = String::new();
        let mut current = self.root.clone();
        let path = path.trim().trim_start_matches('/');
        let mut path_iter = path.split('/');
        let mut next = path_iter.next();
        while let Some(name) = next{
            let child = current.borrow().get_child(name);
            match child{
                Some(child) => current = child,
                None => return None,
            }
            next = path_iter.next();
        }
        Some(current)

    }

    /// Returns none if node is file or if the path is invalid
    async fn get_children(&self, path: &str) -> Option<Vec<NodeRef>>{
        let node = self.get_file(path).await?;
        if node.borrow().is_file{
            return None;
        }
        let children = node.borrow().get_children().await;
        Some(children)
    }

    pub async fn build_catalogue(node: NodeRef) -> Vec<NodeRef>{
        Vec::new()
    }
    
}
