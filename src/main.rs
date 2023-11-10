use std::{path::{PathBuf, Path}, collections::VecDeque, error::Error, cell::{RefCell, RefMut}, rc::Rc, sync::Arc};
use convert::{create_index_document, Document, create_search_index, SearchIndex};
use ctx::{Context};
use fs::{create_target_dir, copy_directory};
use index::{NodeProperty, collect_file_node_recursive, Node, NodeType, FileType, read_node};
use lazy_static::lazy_static;

use clap::Parser;
use template::Template;
use tokenizer::Tokenizer;
use tokio::{fs::File, io::{BufReader, AsyncReadExt, BufWriter, AsyncWriteExt, AsyncBufReadExt}};

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
        let tokenizer = Tokenizer::new(1).unwrap();
        tokenizer
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{

    let head = Node::new(NodeProperty::new("".into(), CONTEXT.config.base.clone().into(), "/".into()).await?, None, Vec::new());
    for target in &CONTEXT.config.target {
        let node = read_node(target.into()).await?;
        head.borrow_mut().children.push(node);
    }
    create_target_dir(head.clone(), create_index_document).await?;
    let files = collect_file_node_recursive(head.clone())?;

    let mut handles = Vec::new();

    for node in files.into_iter() {
        let NodeProperty { node_type, source, mut target, ..} = node.borrow().property.clone();
        if let NodeType::File(FileType::Markdown(Document { property, html, .. })) = node_type {
            let handle = tokio::spawn(async move {

                let f = File::options().read(true).open(source.clone()).await?;             
                let mut reader = BufReader::new(f);
                let mut data = String::new();
                reader.read_to_string(&mut data).await.unwrap();

                let filename_ref = target.file_stem().unwrap().to_str().unwrap();
                let filename = String::from(filename_ref); 
                target.pop();
                target.push(format!("{}.html",filename));


                let mut context = tera::Context::new();
                context.insert("title", &property.title.unwrap_or("undefined".to_string()));
                context.insert("alias", &property.alias.unwrap_or("undefined".to_string()));
                context.insert("author", &property.author.unwrap_or("undefined".to_string()));
                context.insert("created_at", &property.created_at.unwrap_or("undefined".to_string()));
                context.insert("tags", &property.tags.unwrap_or(vec![]));
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
    }
    
    for handle in handles {
        let _ = handle.await?;
    }

    copy_directory(CONTEXT.config.r#static.clone().into(), PathBuf::from(&CONTEXT.config.base).join("static")).await?;
    let search_indices = create_search_index(head.clone())?;
    let exports = search_indices.into_iter().map(|index| {index}).collect::<Vec<SearchIndex>>();
    let binary = bincode::serialize(&exports)?;

    let f = File::options().write(true).create(true).truncate(true).open(PathBuf::from(&CONTEXT.config.base).join("static/searchindex")).await?;
    let mut writer = BufWriter::new(f);
    writer.write(&binary[..]).await?;
    writer.flush().await?;
    Ok(())
}
