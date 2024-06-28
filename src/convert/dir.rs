use std::{ops::Neg, path::PathBuf, str::FromStr, sync::Arc};


use tokio::sync::RwLock;

use crate::{get_context, index::{Node, NodePath, NodeProperty, NodeType}, util::{resolve_osstr_default, resolve_path, sem::Lock}};

use super::{render::{List, Page, Prop}, FileType};

#[derive(Debug)]
pub enum DirType {
    Entry(DirProperty),
    Page(usize, usize), // (index, total)
}
impl DirType {
    pub async fn new(node: &Arc<Node>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        /*
         * !sort_children wait until child node is ready
         */
        sort_children(node).await?;
        
        /*
         * !page_children aquire write lock of (node.path, node.children)
         * !logical becuase sort_children wait until child node finishes it's job (prepare_node
         * must be done first)
         */
        let lk = node.children.read().await;
        let child_node_size = count_valid_children(&lk).await?;
        drop(lk);
        let is_paged = page_children(node, get_context().await.render.list_size).await?;

        /* for child in &*lk {
            println!("{:?}", child.property.node_type);
        } */
        
        /*
         * now all access to node must be done by read mechanism
         */
        Ok(DirType::Entry(DirProperty {
            key: resolve_osstr_default(node.path.read().await.org_path.file_stem())?.into(),
            child_node_size,
            is_paged,
        })) 
    }
}
#[derive(Debug)]
pub struct DirProperty {
    pub key: String,
    pub child_node_size: usize,
    pub is_paged: bool,
}

pub async fn count_valid_children(children: &Vec<Arc<Node>>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let mut count = 0;
    for child in children {
        match &child.property.node_type {
            NodeType::Dir(lk) => {
                match &*lk.read().await {
                    Some(_) => {
                        count += 1;
                    },
                    None => {
                        return Err("Dir not ready panic".into());
                    }
                }
            },
            NodeType::File(lk) => {
                match &*lk.read().await {
                    Some(FileType::Markdown(_, _)) => {
                        count += 1;
                    },
                    None => {
                        return Err("File not ready panic".into());
                    }
                    _ => {}
                }
            }
            
        }
    }
    Ok(count)
}

pub async fn sort_children(node: &Arc<Node>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut children = node.children.write().await;

    /*
     * todo: date descending order
     */

    // ( ( priority, dir<0> or file<1>, timestamp, key ), node )
    let mut sorted_children = Vec::new();

    // wait for children be ready
    for child in &*children {
        let _ = child.property.lk.access("prepare").await?;
        match &child.property.node_type {
            NodeType::Dir(lk) => {
                let lk = lk.read().await;
                match &*lk {
                    Some(DirType::Entry(dp)) => {
                        /*
                         * sort dir in alphabetic order
                         */
                        sorted_children.push(((0, 0, 0, dp.key.clone()), child));
                    },
                    _ => {
                        return Err("locking panic!".into()); 
                    }
                }
            },
            NodeType::File(lk) => {
                let lk = lk.read().await;
                match &*lk {
                    Some(FileType::Markdown(fp, doc)) => {
                        let priority = match doc.property.priority {
                            Some(p) => p as i32,
                            None => 0,
                        };
                        let created_at = match &doc.property.created_at {
                            Some(time) => {
                                let dt = chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00:00", &time), "%Y-%m-%d %H:%M:%S")?.and_utc();
                                dt.timestamp().neg()
                            },
                            None => {0},
                        };
                        sorted_children.push(((priority.neg(), 1, created_at, fp.key.clone()), child));
                    },
                    Some(FileType::Binary(fp)) => {
                        sorted_children.push(((0, 1, 0, fp.key.clone()), child));
                    },
                    _ => return Err("locking panic!".into())
                }
            }
        }
    }
    sorted_children.sort_by_key(|k| k.0.clone());
    let sorted_children = sorted_children.into_iter().map(|(_key, node)| node.clone()).collect::<Vec<Arc<Node>>>();
    *children = sorted_children;

    /* for child in sorted_children {
        println!("{:?}", child.0);
    } */

    Ok(())
}

pub async fn page_children(node: &Arc<Node>, page_size: usize) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut children_lk = node.children.write().await;
    let mut children_left = Vec::new();
    let mut children_target = Vec::new();
    for child in &*children_lk {
        match &child.property.node_type {
            NodeType::File(lk) => {
                let lk = lk.read().await; 
                match &*lk {
                    Some(FileType::Binary(_)) => {
                        children_left.push(child.clone());
                    },
                    _ => {
                        children_target.push(child.clone());
                    }
                }
            },
            _ => {
                children_target.push(child.clone());
            }
        }
    }
    if children_target.len() <= page_size {
        return Ok(false);
    }
    let page_total = (children_target.len()-1) / page_size + 1;

    for (idx, page) in children_target.chunks(page_size).enumerate() {
        let idx = idx + 1;
        let mut rel_path = PathBuf::from_str("./")?;
        rel_path.push(idx.to_string());
        
        let node_path = NodePath {
            org_path: node.path.read().await.org_path.clone(),
            rel_path,
            abs_path: None,
            target_path: None,
        };

        /*
         * manually create page node
         */
        let node = Arc::new(Node {
            property: NodeProperty {
                node_type: NodeType::Dir(RwLock::new(Some(DirType::Page(idx, page_total)))),
                lk: Lock::new(&["prepare"]),
            },
            path: RwLock::new(node_path),
            children: RwLock::new(page.to_vec()),
        });
        node.property.lk.ready("prepare").await?;
        children_left.push(node);
    }
    *children_lk = children_left;
    
    Ok(true)
}

pub async fn convert_render_list(children: &Vec<Arc<Node>>) -> Result<Vec<List>, Box<dyn std::error::Error + Send + Sync>> {
    let mut list = Vec::new();
    for child in children {
        // wait for all children to be ready
        let _ = child.property.lk.access("prepare").await?;
        
        let mut abs_path = match &child.path.read().await.abs_path {
            Some(abs_path) => abs_path.clone(),
            None => return Err("abs_path is not ready".into()),
        };
        
        match &child.property.node_type {
            NodeType::Dir(lk) => {
                match &*lk.read().await {
                    Some(DirType::Entry(dp)) => {
                        if dp.is_paged {
                            abs_path.push("1");
                        }
                        abs_path.push("index.html");
                        list.push(List {
                            link: resolve_path(&abs_path)?.into(),         
                            title: dp.key.clone(),
                            created_at: "".into(),
                            author: "".into(),
                            summary: "".into(),
                            child_node: dp.child_node_size,
                            is_pinned: false,
                            cover_images: vec![],
                        }) 
                    },
                    _ => {}
                }
            },
            NodeType::File(lk) => {
                match &*lk.read().await {
                    Some(FileType::Markdown(fp, doc)) => {
                        /*
                         * waits for image_lk because convert_html of file node writes to image
                         */
                        let _ = doc.parameter.image_lk.access("image").await?;
                        let cover_images = doc.parameter.image.read().await.clone();
                        let cover_images = cover_images.into_iter().map(|url|{ url.replace(' ', r"\ ") }).collect();
                        let summary = match &doc.property.aliases {
                            Some(v) => {
                                v.join(" ")
                            },
                            None => {
                                doc.parameter.summary.clone()
                            }
                        };
                        list.push(List {
                            link: resolve_path(&abs_path)?.into(),
                            title: fp.key.clone(),
                            created_at: doc.property.created_at.clone().unwrap_or("undefined".to_owned()),
                            author: doc.property.author.clone().unwrap_or("undefined".to_owned()),
                            summary,
                            child_node: 0,
                            is_pinned: doc.property.priority.is_some(),
                            cover_images

                        })

                    },
                    None => {
                        return Err("node render panic!: node is not ready".into());
                    },
                    _ => {},
                }
            }
        }
    }
    Ok(list)
}

pub async fn convert_render_page(rel: PathBuf, page: Option<(usize, usize)>) -> Result<(Vec<Page>, Prop), Box<dyn std::error::Error + Send + Sync>> {
    let mut page_indices = Vec::new();
    let mut prop = Prop {paged: false, bottom_href: None, top_href: None};

    match page {
        Some((index, total)) => {
            let indexing_size = 3;
            let mut s: i32 = index as i32 - indexing_size;
            let mut e: i32 = index as i32 + indexing_size;
            if s <= 0 {
                let diff = 1-s;
                s += diff;
                e += diff;
            }
            if e > total as i32 {
                let diff = e - total as i32;
                s -= diff;
                e -= diff;
            }
            if s <= 0 {
                let diff = 1-s;
                s += diff;
            }

            for i in s..e+1 {
                let mut rel = rel.clone();
                rel.pop();
                rel.push(&i.to_string());
                page_indices.push(Page { index: i, cursor: i == index as i32, href: rel.to_str().ok_or("cannot convert path to str")?.to_owned()});
            }

            let mut bottom: i32 = index as i32 - (indexing_size + 1);
            let mut top: i32 = index as i32 + (indexing_size + 1);
            if bottom <= 0 {
                bottom = 1; 
            }
            if top > total as i32 {
                top = total as i32;
            }
            prop.paged = true;
            let mut rel = rel.clone();
            rel.pop();
            rel.push(&bottom.to_string());
            prop.bottom_href = Some(rel.to_str().ok_or("cannot convert path to str")?.to_owned());
            rel.pop();
            rel.push(&top.to_string());
            prop.top_href = Some(rel.to_str().ok_or("cannot convert path to str")?.to_owned());

        },
        None => {}
    }

    Ok((page_indices, prop))
}
