use std::{path::PathBuf, collections::VecDeque, error::Error, rc::Rc, cell::RefCell};

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
    pub fn new(property: NodeProperty, parent: Option<Rc<RefCell<Node>>>, children: Vec<Rc<RefCell<Node>>>) -> Self {
        Self {
            property,
            parent,
            children
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeProperty {
    pub node_type: NodeType,
    pub source: PathBuf,
    pub target: PathBuf,
    pub rel: PathBuf,
    pub document: Option<Document>,
}
impl NodeProperty {
    pub async fn new(source: PathBuf, target: PathBuf, mut rel: PathBuf) -> Result<Self, Box<dyn Error + Send + Sync>> {
        match source.is_dir() {
            true => {
                
                Ok(NodeProperty {
                    node_type: NodeType::Dir,
                    source,
                    target,
                    rel,
                    document: None,
                })
            },
            false => {
                let filename_ref = target.file_stem().unwrap().to_str().unwrap();
                let filename = String::from(filename_ref); 
                rel.pop();
                rel.push(format!("{}.html",filename));
                Ok(NodeProperty {
                    node_type: NodeType::File,
                    source: source.clone(),
                    target,
                    rel,
                    document: Some(Document::new(source).await?),
                }) 
            }
            
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType{
    Dir,
    File,
}

pub async fn read_path_tree(path: PathBuf) -> Result<Rc<RefCell<Node>>, Box<dyn Error + Send + Sync>> {
    let mut target_path = PathBuf::from(&CONTEXT.config.base);
    target_path.push(path.file_stem().unwrap());

    let mut relative_path = PathBuf::from("/");
    relative_path.push(path.file_stem().unwrap());
    
    let head = match &path.is_dir() {
        true => {
            Node {
                property: NodeProperty::new(path, target_path, relative_path).await?,
                parent: None,
                children: Vec::new(),
            }
        },
        false => {
            Node {
                property: NodeProperty::new(path, target_path, relative_path).await?,
                parent: None,
                children: Vec::new(),
            }
        }
    };
    let head = Rc::new(RefCell::new(head));

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
                    let new_node = Node {
                        property: NodeProperty::new(entry.path(), tp, tr).await?, 
                        parent: Some(node.clone()),
                        children: Vec::new(),
                    };
                    let new_node = Rc::new(RefCell::new(new_node));
                    n.children.push(new_node.clone());
                    q.push_back(new_node);
                } 
            },
            NodeType::File => {

            }
        }
    }

    Ok(head)
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
            NodeType::File => {
                files.push(node.clone());
            }
        }
    }

    Ok(files)
}

pub fn print_node(node: Rc<RefCell<Node>>) {
    let node = node.borrow();
    dbg!(&node.property); 
    if !node.children.is_empty() {
        for child in &node.children {
            print_node(child.clone());
        } 
    }
}
