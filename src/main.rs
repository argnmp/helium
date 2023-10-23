use std::{path::{PathBuf, Path}, collections::VecDeque, error::Error, cell::{RefCell, RefMut}, rc::Rc, sync::Arc};
use convert::create_index_document;
use ctx::{Context, ResourceFlag};
use fs::create_target_dir;
use index::{read_path_tree, NodeProperty, collect_file_node_recursive, Node};
use lazy_static::lazy_static;

use clap::Parser;
use template::{Template, TemplateFlag};
use tokio::{fs::{read_dir, create_dir_all, File, copy}, io::{BufReader, AsyncReadExt, BufWriter, AsyncWriteExt, AsyncBufReadExt}};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

mod ctx;
mod fs;
mod index;
mod template;
mod convert;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{

    // let mut files = Vec::new();
    // 가장 head의 cursor은 필요없는 값이다.
    let mut head = Node::new(NodeProperty::Dir(CONTEXT.config.base.clone().into(), CONTEXT.config.base.clone().into(), "/".into()), None, Vec::new());
    for target in &CONTEXT.config.target {
        let node = read_path_tree(target.into()).await?;
        head.children.push(node);
        // files.append(&mut f);
    }
    let head = Rc::new(RefCell::new(head));
    create_target_dir(
        head.clone(),
        create_index_document,
        ).await?;
    let files = collect_file_node_recursive(head.clone())?;

    let mut handles = Vec::new();

    let template = Template::new(CONTEXT.config.resource.get(&ResourceFlag::Layout).unwrap().into()).await?;
    let template = Arc::new(template);

    for node in files.into_iter() {
        let NodeProperty::File(cursor, mut target, _) = node.borrow().property.clone() else {
            continue;
        };
        let template = template.clone(); 
        let handle = tokio::spawn(async move {
            // dbg!(&cursor, &target);

            let f = File::options().read(true).open(cursor).await?;             
            let mut reader = BufReader::new(f);
            let mut data = String::new();
            reader.read_to_string(&mut data).await.unwrap();

            let html = markdown::to_html(&data);

            let filename_ref = target.file_stem().unwrap().to_str().unwrap();
            let filename = String::from(filename_ref); 
            target.pop();
            target.push(format!("{}.html",filename));

            let commit = template.replace(TemplateFlag::Post, &html).await?;
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

async fn copy_directory(from: PathBuf, to: PathBuf) -> Result<(), Box<dyn Error + Sync + Send>> {
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
