use std::{collections::HashSet, hash::DefaultHasher, sync::Arc};

use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use xorf::{HashProxy, Xor16};

use crate::{index::{Node, NodeType}, util::{fs::write_from_slice, resolve_path}};

use super::{file::FileType};

#[derive(Deserialize, Serialize)]
pub struct SearchIndex {
    pub filter: HashProxy<String, DefaultHasher, Xor16>,
    pub title: String,
    pub rel: String,
}
impl SearchIndex {
    fn new(token: &HashSet<String>, title: &str, link: String) -> Self {
        let tokens: Vec<String> = token.clone().into_iter().collect();
        let filter = HashProxy::from(&tokens);
        SearchIndex {
            filter,
            title: title.into(),
            rel: link,
        }
    }
}

#[async_recursion]
pub async fn render_search_index(node: Arc<Node>) -> Result<Vec<SearchIndex>, Box<dyn std::error::Error + Send + Sync>> {
    let mut indices = Vec::new();
    let link = match &node.path.read().await.abs_path {
        Some(abs_path) => resolve_path(abs_path)?.into(),
        None => return Err("abs_path is not ready".into()),
    };

    let path = match &node.path.read().await.target_path {
        Some(target_path) => target_path.clone(),
        None => return Err("abs_path is not ready".into()),
    };

    match &node.property.node_type {
        NodeType::Dir(_) => {
            for child in &*node.children.read().await {
                indices.append(&mut render_search_index(child.clone()).await?);
            }
            let binary = bincode::serialize(&indices)?;
            write_from_slice(&path.join("searchindex"), &binary[..]).await?;
        },
        NodeType::File(lk) => {
            match &*lk.read().await {
                Some(FileType::Markdown(_, doc)) => {
                    let title = match &doc.property.title {
                        Some(title) => title.clone(),
                        None => "undefined".to_owned(),
                    };
                    let search_index = SearchIndex::new(&doc.parameter.token, &title, link);
                    indices.push(search_index);
                },
                _ => {}
            }
        }
    }

    Ok(indices)
}

