use std::{sync::Arc, pin::Pin, future::Future};

use tokio::sync::RwLock;

use crate::{index::{Node, NodeProperty, NodeType, DirType, FileType}, convert::{SearchIndex, self}};

pub fn render_search_index(node: Arc<RwLock<Node>>) -> Pin<Box<dyn Future<Output = Result<Vec<SearchIndex>, Box<dyn std::error::Error + Send + Sync>>>>> {
    
    Box::pin(async move {
        let mut indices = Vec::new();
        let n = node.read().await; 
        let NodeProperty { node_type, target, .. } = &n.property;
        match node_type {
            NodeType::Dir(dir_type) => {
                // dbg!(dir_type);
                for next in &n.children{
                    indices.append(&mut render_search_index(next.clone()).await?);            
                }
                // println!("{} -> indices size: {}", &target.to_str().unwrap(), indices.len());

                let binary = bincode::serialize(&indices)?;
                crate::fs::write_from_slice(&target.join("searchindex"), &binary[..]).await?;

            },
            NodeType::File(FileType::Markdown(_)) => {
                let search_index = convert::create_search_index(node.clone()).await?; 
                indices.push(search_index);
            },
            _ => {}
        }    

        Ok(indices)
    })

}
