use std::{collections::{HashMap, VecDeque}, path::{Path, PathBuf}, str::FromStr, sync::Arc};

use async_recursion::async_recursion;
use tokio::sync::RwLock;
use walkdir::WalkDir;

use crate::{convert::{dir::DirType, file::FileType}, util::{resolve_osstr, resolve_osstr_default, sem::Lock}};

pub struct Node {
    pub property: NodeProperty,
    pub path: RwLock<NodePath>, 
    pub children: RwLock<Vec<Arc<Node>>>,
}
impl Node {
    pub async fn new(node_path: NodePath) -> Node {
        let lk = Lock::new(&["prepare"]);
        let node_type = match node_path.org_path.is_dir() {
            true => {
                NodeType::Dir(RwLock::new(None))
            },
            false => NodeType::File(RwLock::new(None)),
        };
        Node {
            property: NodeProperty { node_type, lk },
            path: RwLock::new(node_path),
            children: RwLock::new(Vec::new()),
        } 
    }
    pub async fn default(node_path: NodePath) -> Node {
        let lk = Lock::new(&["prepare"]);
        let node_type = NodeType::Dir(RwLock::new(None)); 
        Node {
            property: NodeProperty { node_type, lk },
            path: RwLock::new(node_path),
            children: RwLock::new(Vec::new()),
        } 
    }
    async fn format(&self) -> String {
        match &self.property.node_type {
            NodeType::Dir(_) => {
                format!("Dir: {}", self.path.read().await.rel_path.to_str().unwrap())
            },
            NodeType::File(lk) => {
                let lk = lk.read().await;
                let inner = match &*lk {
                    Some(ft) => {
                        match ft {
                            FileType::Markdown(fp, _) => {
                                fp.key.clone()
                            },
                            FileType::Binary(fp) => {
                                fp.key.clone()
                            }
                        } 
                    },
                    None => {
                        "None".into()
                    }
                };
                format!("File: {}", inner)
            }
        }

    }
}

#[derive(Debug)]
pub struct NodeProperty {
    pub node_type: NodeType,
    pub lk: Lock,
}
#[derive(Debug)]
pub enum NodeType {
    Dir(RwLock<Option<DirType>>),
    File(RwLock<Option<FileType>>),
}

pub struct NodePath {
    pub org_path: PathBuf,
    pub rel_path: PathBuf,
    pub abs_path: Option<PathBuf>,
    pub target_path: Option<PathBuf>,
}

pub async fn build_tree(path: &Path) -> Result<Arc<Node>, Box<dyn std::error::Error + Send + Sync>>{
    let dir = WalkDir::new(path); 
    let mut node_stack: Vec<Arc<Node>> = Vec::new();
    let mut depth: usize = 0;
    let web_root = PathBuf::from_str("./")?;

    for entry in dir {
        let entry = entry?;
        
        if entry.depth() <= depth {
            for _ in 0..depth-entry.depth()+1 {
                node_stack.pop();
            }
        }

        let mut rel_path = PathBuf::new();
        /*
         * update depth and rel path
         */
        depth = entry.depth();
        let file_name = match entry.path().is_dir() {
            true => resolve_osstr(entry.path().file_stem())?.to_string(),
            false => {
                match resolve_osstr(entry.path().extension())? {
                    "md" => {
                        resolve_osstr(entry.path().file_stem())?.to_string() + ".html"
                    },
                    _ => resolve_osstr(entry.path().file_name())?.to_string(),
                }
            }
        };
        rel_path.push(&file_name);
        
        let node_path = NodePath {
            org_path: entry.path().into(),
            rel_path: web_root.join(&rel_path),
            abs_path: None,
            target_path: None,
        };

        /*
         * create node with node path
         */
        
        let node = Arc::new(Node::new(node_path).await);
        if let Some(n) = node_stack.last() {
            let mut n = n.children.write().await;
            n.push(node.clone());
        }
        node_stack.push(node); 
    }

    Ok(node_stack.first().ok_or("no node created")?.clone())
}


pub async fn flatten_node(root: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut nodes = Vec::new(); 
    let mut queue = VecDeque::new();
    queue.push_back(root.clone());

    while let Some(node) = queue.pop_front() {
        nodes.push(node.clone()); 
        for node in &*node.children.read().await {
            queue.push_back(node.clone());
        }
    }

    nodes
}
pub async fn flatten_dir_node(root: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut nodes = Vec::new(); 
    let mut queue = VecDeque::new();
    queue.push_back(root.clone());

    while let Some(node) = queue.pop_front() {
        if let NodeType::Dir(_) = node.property.node_type {
            nodes.push(node.clone()); 
        }
        for node in &*node.children.read().await {
            queue.push_back(node.clone());
        }
    }

    nodes
}
pub async fn flatten_file_node(root: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut nodes = Vec::new(); 
    let mut queue = VecDeque::new();
    queue.push_back(root.clone());

    while let Some(node) = queue.pop_front() {
        if let NodeType::File(_) = node.property.node_type {
            nodes.push(node.clone()); 
        }
        for node in &*node.children.read().await {
            queue.push_back(node.clone());
        }
    }

    nodes
}

pub async fn init_remaining_path(root: &Arc<Node>, target_prefix: &Path, collect_documents: &Option<&Path>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let web_root_prefix = PathBuf::from_str("/")?;

    let mut queue = VecDeque::new();
    queue.push_back((root.to_owned(), PathBuf::new()));
    
    while let Some((node, mut path)) = queue.pop_front() {
        path.push(resolve_osstr_default(node.path.read().await.rel_path.file_name())?); 
        if let NodeType::File(lk) = &node.property.node_type {
            if let Some(FileType::Markdown(_, _)) = &*lk.read().await {
                if let Some(collect_path) = collect_documents {
                    // File node has no children. So it's okay to modify path.
                    path = collect_path.to_path_buf();
                    path.push(resolve_osstr_default(node.path.read().await.rel_path.file_name())?);
                }
            }
        }
        let mut path_lk = node.path.write().await;
        path_lk.abs_path = Some(web_root_prefix.join(&path));
        path_lk.target_path = Some(target_prefix.join(&path));
        for node in &*node.children.read().await {
            queue.push_back((node.clone(), path.clone()));
        }
    }

    Ok(())
}



/*
 * collect all resources that can be linked to other resources
 */
pub async fn collect_resource(root: &Arc<Node>) -> Result<HashMap<String, PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut resource_map = HashMap::new();
    let file_nodes = flatten_file_node(root).await;
    for node in file_nodes {
        let key = match &node.property.node_type {
            NodeType::File(lk) => {
                let lk = lk.read().await;
                match &*lk {
                    Some(FileType::Markdown(p, _)) => {
                        p.key.clone()       
                    },
                    Some(FileType::Binary(p)) => {
                        p.key.clone()
                    },
                    None => {
                        return Err("collect resource panic".into());
                    }
                }
                
            },
            _ => {
                return Err("collect reource panic: node type dir".into());
            }
        };
        match &node.path.read().await.abs_path {
            Some(abs_path) => {
                resource_map.insert(key, abs_path.to_owned());
            },
            None => {
                return Err("abs_path not ready".into());
            }

        }
    }
    

    Ok(resource_map)
}

#[allow(dead_code)]
#[async_recursion]
pub async fn print_tree(node: Arc<Node>, depth: usize) {
    for _ in 0..depth {
        print!(" ");
    }
    println!("{}, abs_path: {:?}, target_path: {:?}", node.format().await, node.path.read().await.abs_path, node.path.read().await.target_path);     
    let children = node.children.read().await;
    for child in &*children {
        print_tree(child.clone(), depth + 4).await;
    }
}

