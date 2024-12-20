use std::fs::{self, Metadata};
use std::path::Path;
use std::{collections::HashMap, sync::Arc};

enum FileBuildErr{
    IoErr(std::io::Error),
    PermissionErr(String),
    NotFoundErr(String),
    InvalidPath(String),
    IsSymlink(String),
}

struct Node{
    name: String,
    children: HashMap<String, Arc<Node>>,
    parent: Option<Arc<Node>>,
    is_file: bool,
    size: usize,
}

struct FileTree{
    root: Arc<Node>,
    full_path: Vec<String>,
    current: Arc<Node>,
    size: usize,

}

impl Node{
    fn new(name: String, is_file: bool, size: usize) -> Node{
        Node{
            name,
            children: HashMap::new(),
            parent: None,
            is_file,
            size,
        }
    }

    fn add_child(&mut self, child: Node){
        self.children.insert(child.name.clone(), Arc::new(child));
    }

    fn get_child(&self, name: &str) -> Option<Arc<Node>>{
        self.children.get(name).cloned()
    }

    fn get_children(&self) -> Vec<Arc<Node>>{
        self.children.values().map(|x| x.clone()).collect()
    }

    fn from_file(path_str: &str, parent: Option<Arc<Node>>) -> Result<Node, FileBuildErr>{
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
        if metadata.is_symlink(){
            return Err(FileBuildErr::IsSymlink(path_str.to_string()));
        }
        let size = metadata.len() as usize;

        Ok(Node{
            name,
            children: HashMap::new(),
            parent,
            is_file,
            size,
        })
        
    }
}

impl FileTree{
    fn check_metadata<T>(path: T) -> Result<Metadata, FileBuildErr>
    where T: Into<String> + AsRef<Path>
    {
        let metadata = match fs::metadata(path.as_ref()){
            Ok(meta) => meta,
            Err(e) => match e.kind(){
                std::io::ErrorKind::PermissionDenied => return Err(FileBuildErr::PermissionErr(path.into())),
                std::io::ErrorKind::NotFound => return Err(FileBuildErr::NotFoundErr(path.into())),
                _ => return Err(FileBuildErr::IoErr(e)),
            },
        };
        if metadata.is_symlink(){
            return Err(FileBuildErr::IsSymlink(path.into()));
        }

        if path.as_ref().file_name() == None{
            return Err(FileBuildErr::InvalidPath(path.into()));
        }   

        Ok(metadata)
    }
    
    pub fn from_root_dir(path: &str) -> Result<Self, FileBuildErr>{
        let root = match Node::from_file(path, None){
            Ok(node) => node,
            Err(e) => return Err(e),
        };
        let root_dir = match fs::read_dir(path){
            Ok(dir) => dir,
            Err(e) => match e.kind(){
                std::io::ErrorKind::NotFound => return Err(FileBuildErr::NotFoundErr(path.to_string())),
                std::io::ErrorKind::PermissionDenied => return Err(FileBuildErr::PermissionErr(path.to_string())),
                _ => return Err(FileBuildErr::IoErr(e)),
                
            }
        };

        //BOOKMARK :)
        for entry in root_dir{
            let entry = match entry{
                Ok(entry) => entry,
                Err(e) => return Err(FileBuildErr::IoErr(e)),
            };
            let metadata = entry.metadata()
            let name = entry.file_name().to_string_lossy().to_string();
            

        }




    }
}