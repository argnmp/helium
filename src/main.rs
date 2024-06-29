use std::{path::Path, sync::Arc, time::Instant};

use clap::Parser;
use context::{Args, Context};
use convert::{render::get_template, search::render_search_index};
use index::{init_remaining_path, Node};
use tokio::{fs::create_dir_all, sync::OnceCell, task::JoinHandle};
use util::fs::{copy_recursive, remove_dir, write_from_slice};

mod context;
mod index;
mod convert;
mod util;


static CONTEXT: OnceCell<Context> = OnceCell::const_new();
async fn get_context() -> &'static Context {
    CONTEXT.get_or_init(|| async {
        let args = Args::parse();
        
        Context::new(&args.config).expect("context loading failed")
    }).await
}

#[tokio::main(flavor="multi_thread", worker_threads=16)]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let start_time = Instant::now();

    /*
     * test global state setting
     */
    let context = get_context().await;
    let _template = get_template().await;
    // println!("test global state setting: {:?}", start_time.elapsed());

    /*
     * build tree
     */
    let root = Arc::new(Node::default(index::NodePath { org_path: "/".into(), rel_path: "./".into(), abs_path: Some("/".into()), target_path: None }).await);
    let mut children = root.children.write().await;
    for node in &context.nodes {
        let tree = index::build_tree(node).await?;
        children.push(tree);

    }
    drop(children);
    // println!("build tree: {:?}", start_time.elapsed());

    /*
     * prepare data by parsing each node
     */
    let nodes = index::flatten_node(&root).await;
    let mut handles = vec![];
    for node in &nodes {
        let node = node.clone();
        let handle: JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> = tokio::spawn(async move {
            convert::prepare_node(&node).await?;
            Ok(())
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await??;
    }
    // println!("prepare node: {:?}", start_time.elapsed());


    /*
     * init target_path
     */
    let collect_documents = match context.render.collect_documents {
        true => Some(Path::new("post")),
        false => None,
    };
    init_remaining_path(&root, &context.target_base, &collect_documents).await.unwrap();
    // index::print_tree(root.clone(), 0).await;
    // println!("init target path: {:?}", start_time.elapsed());


    /*
     * collect resources for rendering
     */
    let resource_map = Arc::new(index::collect_resource(&root).await.unwrap());

    /*
     * remove data in target_path and copy static files
     */
    remove_dir(&context.target_base, false).await.unwrap();
    let mut static_dir = context.target_base.clone();
    static_dir.push("static");
    for path in &context.render.r#static {
        copy_recursive(path, &static_dir, false).await.unwrap();
    }
    // println!("collect resource & remove and copy files: {:?}", start_time.elapsed());

    /*
     * create directories
     */
    let dir_nodes = index::flatten_dir_node(&root).await;
    let mut handles = vec![];
    for node in &dir_nodes {
        let node = node.clone();
        let handle: JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> = tokio::spawn(async move {
            match &node.path.read().await.target_path {
                Some(target_path) => {
                    create_dir_all(target_path).await?;
                },
                None => {
                    panic!();
                }
            }
            Ok(())
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await??;
    }
    if let Some(path) = collect_documents {
        create_dir_all(context.target_base.join(path)).await?;
    }
    // println!("create directories: {:?}", start_time.elapsed());
    

    /*
     * render nodes
     */
    let nodes = index::flatten_node(&root).await;
    let mut handles = vec![];
    for node in &nodes {
        let node = node.clone();
        let resource_map = resource_map.clone();
        let handle: JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> = tokio::spawn(async move {
            convert::render_node(&node, resource_map).await?;
            Ok(())
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await??;
    }
    // println!("render node: {:?}", start_time.elapsed());

    /*
     * render search indices
     */
    let indices = render_search_index(root).await.unwrap();
    if let Some(path) = collect_documents {
        /*
         * export search index to post directory
         */
        let binary = bincode::serialize(&indices)?;
        write_from_slice(&context.target_base.join(path).join("searchindex"), &binary[..]).await?;
    }
    
    println!("total elapsed: {:?}", start_time.elapsed());

    Ok(())

}
