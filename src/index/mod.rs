use std::{path::PathBuf, collections::VecDeque, error::Error, rc::Rc, cell::RefCell};

use tokio::fs::{create_dir_all, read_dir};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::CONTEXT;

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
pub enum NodeProperty{
    Dir(PathBuf, PathBuf, PathBuf),
    File(PathBuf, PathBuf, PathBuf),
}
impl NodeProperty {
    fn path(cursor: PathBuf, target: PathBuf, mut relative: PathBuf) -> Self {
        match cursor.is_dir() {
            true => {
                NodeProperty::Dir(cursor, target, relative) 
            },
            false => {
                let filename_ref = target.file_stem().unwrap().to_str().unwrap();
                let filename = String::from(filename_ref); 
                relative.pop();
                relative.push(format!("{}.html",filename));
                NodeProperty::File(cursor, target, relative) 
            }
            
        }
    }
}

pub async fn read_path_tree(path: PathBuf) -> Result<Rc<RefCell<Node>>, Box<dyn Error + Send + Sync>> {
    let mut target_path = PathBuf::from(&CONTEXT.config.base);
    target_path.push(path.file_stem().unwrap());

    let mut relative_path = PathBuf::from("/");
    relative_path.push(path.file_stem().unwrap());
    
    let head = match &path.is_dir() {
        true => {
            Node {
                property: NodeProperty::Dir(path, target_path, relative_path),
                parent: None,
                children: Vec::new(),
            }
        },
        false => {
            Node {
                property: NodeProperty::File(path, target_path, relative_path),
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
        let p = n.property.clone();
        match p {
            NodeProperty::Dir(cursor, target, relative) => {
                create_dir_all(&target).await?;
                let mut stream = ReadDirStream::new(read_dir(&cursor).await?);
                while let Some(entry) = stream.next().await {
                    let entry = entry?;
                    let mut tp = target.clone();
                    tp.push(entry.file_name());
                    let mut tr = relative.clone();
                    tr.push(entry.file_name());
                    let new_node = Node {
                        property: NodeProperty::path(entry.path(), tp, tr), 
                        parent: Some(node.clone()),
                        children: Vec::new(),
                    };
                    let new_node = Rc::new(RefCell::new(new_node));
                    n.children.push(new_node.clone());
                    q.push_back(new_node);
                } 
            },
            NodeProperty::File(_, _, _) => {

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
        match &n.property {
            NodeProperty::Dir(_, target, _) => {
                for t in &n.children {
                    q.push_back(t.clone());
                }
            },
            NodeProperty::File(_, _, _) => {
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
