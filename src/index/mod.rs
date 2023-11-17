use std::{path::PathBuf, collections::VecDeque, error::Error, rc::Rc, cell::RefCell, ffi::OsStr, sync::Arc, time::SystemTime};

use tokio::{fs::{create_dir_all, read_dir}, sync::RwLock};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::{CONTEXT, convert::Document};

#[derive(Debug)]
pub struct Node {
    pub property: NodeProperty,
    pub parent: Option<Arc<RwLock<Node>>>,
    pub children: Vec<Arc<RwLock<Node>>>,
}
impl Node {
    pub fn new(property: NodeProperty, parent: Option<Arc<RwLock<Node>>>, children: Vec<Arc<RwLock<Node>>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            property,
            parent,
            children
        }))
    }
}

#[derive(Debug, Clone)]
pub struct NodeProperty {
    pub node_type: NodeType,
    pub source: PathBuf,
    pub target: PathBuf,
    pub rel: PathBuf,
    pub created: SystemTime,
}
impl NodeProperty {
    pub async fn new(source: PathBuf, target: PathBuf, rel: PathBuf) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let metadata = tokio::fs::metadata(&source).await?;
        match source.is_dir() {
            true => {
                
                Ok(NodeProperty {
                    node_type: NodeType::Dir,
                    source,
                    target,
                    rel,
                    created: metadata.created().unwrap_or(SystemTime::now()),
                })
            },
            false => {
                Ok(NodeProperty {
                    node_type: match target.extension().ok_or("extension error")?.to_str().ok_or("no str in osstr")? {
                        "md" => {
                            NodeType::File(FileType::Markdown(Document::new(source.clone()).await?))
                        },
                        "jpeg" | "jpg" | "png" => {
                            NodeType::File(FileType::Binary) 
                        },
                        _ => {
                            return Err("invalid file in directory".into());
                        }
                    },
                    source,
                    target,
                    rel,
                    created: metadata.created().unwrap_or(SystemTime::now()),
                }) 
            }
            
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType{
    Dir,
    File(FileType),
}

#[derive(Debug, Clone)]
pub enum FileType{
    Markdown(Document),
    Binary
}

pub async fn read_node(path: PathBuf) -> Result<Arc<RwLock<Node>>, Box<dyn Error + Send + Sync>> {
    let mut target_path = PathBuf::from(&CONTEXT.config.base);
    target_path.push(path.file_stem().unwrap());

    let mut relative_path = PathBuf::from("/");
    relative_path.push(path.file_stem().unwrap());
    
    let head = Node::new(NodeProperty::new(path, target_path, relative_path).await?, None, Vec::new());

    let mut q: VecDeque<Arc<RwLock<Node>>> = VecDeque::new();
    q.push_back(head.clone());

    while let Some(node) = q.pop_front() {
        let mut n = node.write().await;
        let NodeProperty { node_type, source, target, rel, ..} = n.property.clone();
        match node_type {
            NodeType::Dir => {
                let mut stream = ReadDirStream::new(read_dir(&source).await?);
                while let Some(entry) = stream.next().await {
                    let entry = entry?;
                    
                    let mut tp = target.clone();
                    tp.push(entry.file_name());
                    let mut tr = rel.clone();
                    tr.push(entry.file_name());

                    let new_node = Node::new(NodeProperty::new(entry.path(), tp, tr).await?, Some(node.clone()), Vec::new());
                    n.children.push(new_node.clone());
                    q.push_back(new_node);
                } 
            },
            NodeType::File(_) => {

            }
        }
    }

    Ok(head)
}

pub async fn flatten_node(node: Arc<RwLock<Node>>) -> Result<Vec<Arc<RwLock<Node>>>, Box<dyn Error + Sync + Send>> {
    // directories are always put in front of documents inside
    let mut nodes = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(node);
    while let Some(node) = queue.pop_front() {
        nodes.push(node.clone()); 
        let node = node.read().await;
        let NodeProperty { node_type, .. } = &node.property;
        match node_type {
            NodeType::Dir => {
                for next_node in &node.children {
                    queue.push_back(next_node.clone());
                }
            },
            NodeType::File(_) => {}
        }
    }
    Ok(nodes)
}
pub async fn flatten_dir_node(node: Arc<RwLock<Node>>) -> Result<Vec<Arc<RwLock<Node>>>, Box<dyn Error + Sync + Send>> {
    // directories are always put in front of documents inside
    let mut nodes = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(node);
    while let Some(node) = queue.pop_front() {
        let n = node.read().await;
        let NodeProperty { node_type, .. } = &n.property;
        match node_type {
            NodeType::Dir => {
                for next_node in &n.children {
                    queue.push_back(next_node.clone());
                }
                drop(n);
                nodes.push(node);
            },
            NodeType::File(_) => {
            }
        }
    }
    Ok(nodes)
}
pub async fn flatten_file_node(node: Arc<RwLock<Node>>) -> Result<Vec<Arc<RwLock<Node>>>, Box<dyn Error + Sync + Send>> {
    // directories are always put in front of documents inside
    let mut nodes = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(node);
    while let Some(node) = queue.pop_front() {
        let n = node.read().await;
        let NodeProperty { node_type, .. } = &n.property;
        match node_type {
            NodeType::Dir => {
                for next_node in &n.children {
                    queue.push_back(next_node.clone());
                }
            },
            NodeType::File(_) => {
                nodes.push(node.clone());
            }
        }
    }
    Ok(nodes)
}


pub async fn collect_file_node_recursive(node: Arc<RwLock<Node>>) -> Result<Vec<Arc<RwLock<Node>>>, Box<dyn Error + Sync + Send>> {
    let mut files = Vec::new(); 
    let mut q: VecDeque<Arc<RwLock<Node>>> = VecDeque::new();
    q.push_back(node);
    
    while let Some(node) = q.pop_front() {
        let n = node.read().await;
        let NodeProperty { node_type, .. } = &n.property;
        match node_type {
            NodeType::Dir => {
                for t in &n.children {
                    q.push_back(t.clone());
                }
            },
            NodeType::File(_) => {
                files.push(node.clone());
            }
        }
    }

    Ok(files)
}
