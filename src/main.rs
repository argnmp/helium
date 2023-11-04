use std::{path::{PathBuf, Path}, collections::VecDeque, error::Error, cell::{RefCell, RefMut}, rc::Rc, sync::Arc};
use convert::{create_index_document, Document};
use ctx::{Context, ResourceFlag};
use fs::{create_target_dir, copy_directory};
use index::{read_path_tree, NodeProperty, collect_file_node_recursive, Node};
use lazy_static::lazy_static;

use clap::Parser;
use template::Template;
use tokio::{fs::File, io::{BufReader, AsyncReadExt, BufWriter, AsyncWriteExt, AsyncBufReadExt}};

mod ctx;
mod fs;
mod index;
mod template;
mod convert;
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{

    // 가장 head의 source은 필요없는 값이다.
    let mut head = Node::new(NodeProperty::new(CONTEXT.config.base.clone().into(), CONTEXT.config.base.clone().into(), "/".into()).await?, None, Vec::new());
    for target in &CONTEXT.config.target {
        let node = read_path_tree(target.into()).await?;
        head.children.push(node);
    }
    let head = Rc::new(RefCell::new(head));
    create_target_dir(head.clone(), create_index_document).await?;
   let files = collect_file_node_recursive(head.clone())?;

    let mut handles = Vec::new();

    for node in files.into_iter() {
        let NodeProperty { source, mut target, document, ..} = node.borrow().property.clone();
        let handle = tokio::spawn(async move {

            let f = File::options().read(true).open(source.clone()).await?;             
            let mut reader = BufReader::new(f);
            let mut data = String::new();
            reader.read_to_string(&mut data).await.unwrap();
            
            let document = document.ok_or("incorrect md file")?;
            let html = markdown::to_html(&document.body);

            let filename_ref = target.file_stem().unwrap().to_str().unwrap();
            let filename = String::from(filename_ref); 
            target.pop();
            target.push(format!("{}.html",filename));

            let mut context = tera::Context::new();
            context.insert("title", &document.property.title.unwrap_or("undefined".to_string()));
            context.insert("alias", &document.property.alias.unwrap_or("undefined".to_string()));
            context.insert("author", &document.property.author.unwrap_or("undefined".to_string()));
            context.insert("created_at", &document.property.created_at.unwrap_or("undefined".to_string()));
            context.insert("tags", &document.property.tags.unwrap_or(vec![]));
            context.insert("post", &html);

            let commit = TEMPLATE.tera.render("post.html", &context).unwrap();
            let f = File::options().write(true).create(true).open(target).await.unwrap();
            let mut writer = BufWriter::new(f);
            writer.write(commit.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();

            Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
        });
        handles.push(handle);

    }
    for handle in handles {
        let _ = handle.await?;
    }

    copy_directory(CONTEXT.config.r#static.clone().into(), PathBuf::from("static")).await?;
    Ok(())
}
