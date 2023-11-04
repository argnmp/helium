use std::{error::Error, cell::{RefCell, RefMut}, rc::Rc, collections::VecDeque, future::Future, path::{Path, PathBuf}};

use tokio::fs::{create_dir_all, read_dir, copy};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::{index::{Node, NodeProperty, NodeType}, CONTEXT};

pub async fn create_target_dir<F, Fut>(node: Rc<RefCell<Node>>, node_callback: F) -> Result<(), Box<dyn Error + Send + Sync>> where F: Fn(Rc<RefCell<Node>>) -> Fut, Fut: Future<Output=Result<(), Box<dyn Error + Send + Sync>>>{
    let mut q: VecDeque<Rc<RefCell<Node>>> = VecDeque::new();
    q.push_back(node);
    
    while let Some(node) = q.pop_front() {
        let n = node.borrow();
        let NodeProperty {node_type, target, ..} = &n.property;
        node_callback(node.clone()).await?;
        match node_type {
            NodeType::Dir => {
                create_dir_all(target).await?;
                
                for t in &n.children {
                    q.push_back(t.clone());
                }
            },
            NodeType::File => {
            }
        }
    }
    
    Ok(())
}

pub async fn copy_directory(from: PathBuf, to: PathBuf) -> Result<(), Box<dyn Error + Sync + Send>> {
    let target_path = PathBuf::from(&CONTEXT.config.base).join(to);
    
    let start_entries = PathBuf::from(from);
    let mut q: VecDeque<(PathBuf, PathBuf)> = VecDeque::new();
    q.push_back((start_entries, target_path));
    
    while let Some((cursor, target)) = q.pop_front() {
        match cursor.is_dir() {
            true => {
                create_dir_all(&target).await?;
                let mut stream = ReadDirStream::new(read_dir(&cursor).await?);
                while let Some(entry) = stream.next().await {
                    let entry = entry?;
                    let mut tp = target.clone();
                    tp.push(entry.file_name());

                    q.push_back((entry.path(), tp));
                } 
            },
            false => {
                copy(&cursor, &target).await?;
            }
        }
    }
    Ok(())
     
}
