use std::{collections::HashMap, path::{PathBuf}, sync::Arc};

use crate::{convert::render::create_post_page, index::{Node, NodeType}, util::{fs::copy_file, resolve_path}};

use self::{dir::{convert_render_list, convert_render_page, DirType}, file::{convert_html, FileType}, render::create_index_page};

pub mod dir;
pub mod file;
/*
 * uses tera template engine for rendering
 */
pub mod render;
/*
 * search index creation
 */
pub mod search;




/*
 * prepare node of rendering by parsing the metadata
 */
pub async fn prepare_node(node: &Arc<Node>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match &node.property.node_type {
        NodeType::Dir(lk) => {
            let dir_type = DirType::new(node).await?;
            let mut lk = lk.write().await;
            *lk = Some(dir_type);
            
        },
        NodeType::File(lk) => {
            let file_type = FileType::new(node).await?;

            let mut lk = lk.write().await;
            *lk = Some(file_type);
        }
    }
    // lock ready
    node.property.lk.ready("prepare").await?;

    Ok(())
}

pub async fn render_node(node: &Arc<Node>, resource_map: Arc<HashMap<String, PathBuf>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path_lk = node.path.read().await;
    let mut abs_path = match &node.path.read().await.abs_path {
        Some(abs_path) => abs_path.clone(),
        None => return Err("abs path not initialized".into())
    };
    let mut target_path = match &node.path.read().await.target_path {
        Some(target_path) => target_path.clone(),
        None => return Err("target path not initialized".into())
    };
    match &node.property.node_type {
        NodeType::Dir(lk) => {
            let children = node.children.read().await; 
            let list = convert_render_list(&children).await?;
            let (page_indices, prop) = match &*lk.read().await {
                Some(DirType::Entry(_)) => {
                    convert_render_page(resolve_path(&abs_path)?.into(), None).await?
                },
                Some(DirType::Page(index, total)) => {
                    convert_render_page(resolve_path(&abs_path)?.into(), Some((*index, *total))).await?
                },
                None => {
                    return Err("Dir not ready".into());
                }
            };
            let refresh = match &*lk.read().await {
                Some(DirType::Entry(dp)) => {
                    if dp.is_paged {
                        abs_path.push("1");
                        Some(abs_path)
                    }
                    else {
                        None
                    }
                },
                _ => {
                    None
                }
            };

            target_path.push("index.html");
            create_index_page(&target_path, refresh, &list, &page_indices, &prop).await?;
        },
        NodeType::File(lk) => {
            match &*lk.read().await {
                Some(FileType::Markdown(_, doc)) => {
                    let html = convert_html(doc, resource_map).await?;
                    create_post_page(&target_path, &html, &doc.property).await?;
                },
                Some(FileType::Binary(_)) => {
                    copy_file(&path_lk.org_path, &target_path).await?;
                },
                None => {
                    return Err("File not ready".into());
                }
            }
        }
    }
    Ok(())
}
