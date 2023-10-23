use std::{error::Error, cell::{RefCell, RefMut}, rc::Rc, collections::VecDeque, future::Future, path::Path};

use tokio::fs::create_dir_all;

use crate::index::{Node, NodeProperty};

pub async fn create_target_dir<F, Fut>(node: Rc<RefCell<Node>>, node_callback: F) -> Result<(), Box<dyn Error + Send + Sync>> where F: Fn(Rc<RefCell<Node>>) -> Fut, Fut: Future<Output=Result<(), Box<dyn Error + Send + Sync>>>{
    let mut q: VecDeque<Rc<RefCell<Node>>> = VecDeque::new();
    q.push_back(node);
    
    while let Some(node) = q.pop_front() {
        let n = node.borrow();
        node_callback(node.clone()).await?;
        match &n.property {
            NodeProperty::Dir(_, target, _) => {
                create_dir_all(target).await?;
                
                for t in &n.children {
                    q.push_back(t.clone());
                }
            },
            NodeProperty::File(_, _, _) => {
            }
        }
    }
    
    Ok(())
}
