use std::{path::PathBuf, collections::VecDeque, error::Error, rc::Rc, cell::RefCell, ffi::OsStr};

use tokio::fs::{create_dir_all, read_dir};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::{CONTEXT, convert::Document};

#[derive(Debug)]
pub struct Node {
    pub property: NodeProperty,
    pub parent: Option<Rc<RefCell<Node>>>,
    pub children: Vec<Rc<RefCell<Node>>>,
}
impl Node {
    pub fn new(property: NodeProperty, parent: Option<Rc<RefCell<Node>>>, children: Vec<Rc<RefCell<Node>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
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
}
impl NodeProperty {
    pub async fn new(source: PathBuf, target: PathBuf, rel: PathBuf) -> Result<Self, Box<dyn Error + Send + Sync>> {
        match source.is_dir() {
            true => {
                
                Ok(NodeProperty {
                    node_type: NodeType::Dir,
                    source,
                    target,
                    rel,
                })
            },
            false => {
                Ok(NodeProperty {
                    node_type: match target.extension().ok_or("extension error")?.to_str().ok_or("no str in osstr")? {
                        "md" => {
                            NodeType::File(FileType::Markdown(Document::new(source.clone()).await?))
                        },
                        "jpeg" | "jpg" => {
                            NodeType::File(FileType::Binary) 
                        },
                        _ => {
                            return Err("invalid file in directory".into());
                        }
                    },
                    source,
                    target,
                    rel,
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

pub async fn read_node(path: PathBuf) -> Result<Rc<RefCell<Node>>, Box<dyn Error + Send + Sync>> {
    let mut target_path = PathBuf::from(&CONTEXT.config.base);
    target_path.push(path.file_stem().unwrap());

    let mut relative_path = PathBuf::from("/");
    relative_path.push(path.file_stem().unwrap());
    
    let head = Node::new(NodeProperty::new(path, target_path, relative_path).await?, None, Vec::new());

    let mut q: VecDeque<Rc<RefCell<Node>>> = VecDeque::new();
    q.push_back(head.clone());

    while let Some(node) = q.pop_front() {
        let mut n = node.borrow_mut();
        let NodeProperty { node_type, source, target, rel, ..} = n.property.clone();
        match node_type {
            NodeType::Dir => {
                create_dir_all(&target).await?;
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

pub fn flatten_node(node: Rc<RefCell<Node>>) -> Result<Vec<Rc<RefCell<Node>>>, Box<dyn Error + Sync + Send>> {
    let mut nodes = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(node);
    while let Some(node) = queue.pop_front() {
        nodes.push(node.clone()); 
        let node = node.borrow();
        let NodeProperty { node_type, .. } = &node.property;
        match node_type {
            NodeType::Dir => {
                for next_node in &node.children {
                    nodes.push(next_node.clone());
                }
            },
            NodeType::File(_) => {}
        }
    }
    Ok(nodes)
}

pub fn collect_file_node_recursive(node: Rc<RefCell<Node>>) -> Result<Vec<Rc<RefCell<Node>>>, Box<dyn Error + Sync + Send>> {
    let mut files = Vec::new(); 
    let mut q: VecDeque<Rc<RefCell<Node>>> = VecDeque::new();
    q.push_back(node);
    
    while let Some(node) = q.pop_front() {
        let n = node.borrow_mut();
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

/* pub fn print_node(node: Rc<RefCell<Node>>) {
    let node = node.borrow();
    dbg!(&node.property); 
    if !node.children.is_empty() {
        for child in &node.children {
            print_node(child.clone());
        } 
    }
} */
