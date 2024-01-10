use std::{path::{PathBuf}, collections::{HashMap}, error::Error, sync::Arc};
use convert::{Document};
use ctx::{Context};
use fs::{read_filename, read_filename_with_ext};
use index::{NodeProperty, Node, NodeType, FileType, read_node, flatten_node, flatten_file_node, flatten_dir_node, DirType};
use lazy_static::lazy_static;

use clap::Parser;
use template::Template;
use tokenizer::Tokenizer;
use tokio::{fs::{create_dir_all}, io::{AsyncReadExt, AsyncWriteExt}, sync::{RwLock, Mutex}};

mod ctx;
mod fs;
mod index;
mod template;
mod convert;
mod tokenizer;
mod error;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    config: String, 
}

lazy_static! {
    static ref CONTEXT: Context = {
        let args = Args::parse();
        let ctx = ctx::Context::new(args.config.into()).unwrap();
        ctx 
    };
    static ref TEMPLATE: Template = {
        let mut t = Template::new(CONTEXT.config.template.clone().into()).unwrap();
        t.tera.autoescape_on(vec![]);
        t
    };
    static ref TOKENIZER: Tokenizer = {
        let tokenizer = Tokenizer::new(10).unwrap();
        tokenizer
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{

    let head = Node::new(NodeProperty::new(CONTEXT.config.base.clone().into(), CONTEXT.config.base.clone().into(), "/".into()).await?, None, Vec::new());
    for target in &CONTEXT.config.target {
        let node = read_node(target.into()).await?;
        head.write().await.children.push(node);
    }
    /*
     * create directories
     * this task should be done first to process creating documents in parallel
     */
    let flatten_dir_nodes = flatten_dir_node(head.clone()).await?;
    for node in flatten_dir_nodes {
        let node = &node.read().await;
        let NodeProperty { target, .. } = &node.property;
        create_dir_all(target).await?;
    }

    /*
     * collect links
     */
    let mut included_files = HashMap::new();
    let flatten_file_nodes = flatten_file_node(head.clone()).await?;
    for node in flatten_file_nodes {
        let node = node.read().await;
        let NodeProperty { node_type, target, rel, .. } = &node.property;
        match node_type {
            NodeType::File(FileType::Markdown(_)) => {
                let filename = read_filename(target).await?;
                let mut rel = rel.clone();
                rel.pop();
                rel.push(format!("{}.html",filename));
                
                included_files.insert(read_filename(target).await?, rel);
            },
            NodeType::File(FileType::Binary) => {
                included_files.insert(read_filename_with_ext(target).await?, rel.clone());
            },
            _ => {}
        }
    }

    /*
     * start rendering website
     */
    let included_files = Arc::new(included_files);
    let search_indices = Arc::new(RwLock::new(Vec::new()));
    let mut handles = Vec::new();
    let flatten_nodes = flatten_node(head.clone()).await?;
    let before_task_n = flatten_nodes.len();
    let after_task_n = Arc::new(Mutex::new(0));
    for node in flatten_nodes.into_iter() {
        let included_files = included_files.clone();
        let after_task_n = after_task_n.clone();
        let search_indices = search_indices.clone();
        let handle = tokio::spawn(async move {
            let n = node.read().await;
            let NodeProperty { node_type, source, target,  .. } = &n.property;
            match node_type {
                NodeType::Dir(dir_type) => {
                    match dir_type {
                        DirType::Default(_, is_paged) if *is_paged => { },
                        _ => {
                            convert::create_index_document(node.clone()).await?;
                        }
                    }
                },
                NodeType::File(FileType::Markdown(document)) => {
                    document.prepare_html(included_files).await?.prepare_token().await?;
                    let Document { property,  html,  .. } = document;

                    let filename = fs::read_filename(&target).await?;
                    let mut target = target.clone();
                    target.pop();
                    target.push(format!("{}.html", filename));


                    let property = property.clone();
                    let mut context = tera::Context::new();
                    context.insert("title", &property.title.unwrap_or("undefined".to_string()));
                    context.insert("alias", &property.alias.unwrap_or("undefined".to_string()));
                    context.insert("author", &property.author.unwrap_or("undefined".to_string()));
                    context.insert("created_at", &property.created_at.unwrap_or("undefined".to_string()));
                    context.insert("tags", &property.tags.unwrap_or(vec![]));
                    context.insert("post", &*html.lock().await);

                    let commit = TEMPLATE.tera.render("post.html", &context).unwrap();
                    fs::write_from_string(&target, commit).await?;

                    let search_index = convert::create_search_index(node.clone()).await?;
                    search_indices.write().await.push(search_index);

                },
                NodeType::File(_) => {
                    fs::copy_recursive(&source, &target).await?; 
                }
            }
            *after_task_n.lock().await += 1;
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let _ = handle.await??;
    }
    
    if before_task_n != *after_task_n.lock().await {
        panic!("before_task_n and after_task_n does not match");
    }

    fs::copy_recursive(&PathBuf::from(&CONTEXT.config.r#static), &PathBuf::from(&CONTEXT.config.base).join("static")).await?;
    // let search_indices = create_search_index(head.clone())?;
    // let exports = search_indices.into_iter().map(|index| {index}).collect::<Vec<SearchIndex>>();
    let search_indices = search_indices.write().await;
    let binary = bincode::serialize(&*search_indices)?;
    fs::write_from_slice(&PathBuf::from(&CONTEXT.config.base).join("static/searchindex"), &binary[..]).await?;

    Ok(())
}
