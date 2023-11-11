use std::{error::Error, cell::{RefCell, RefMut}, rc::Rc, collections::VecDeque, future::Future, path::{Path, PathBuf}, sync::Arc};

use tokio::{fs::{create_dir_all, read_dir, copy, File}, sync::Mutex, io::{BufReader, AsyncReadExt, BufWriter, AsyncWriteExt}};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::{index::{Node, NodeProperty, NodeType}, CONTEXT};

/* pub async fn create_target_dir<F, Fut>(node: Arc<Mutex<Node>>, node_callback: F) -> Result<(), Box<dyn Error + Send + Sync>> where F: Fn(Arc<Mutex<Node>>) -> Fut, Fut: Future<Output=Result<(), Box<dyn Error + Send + Sync>>>{

    let mut q: VecDeque<Arc<Mutex<Node>>> = VecDeque::new();
    q.push_back(node);
    
    while let Some(node) = q.pop_front() {
        let n = node.lock().await;
        let NodeProperty {node_type, target, ..} = &n.property;
        node_callback(node.clone()).await?;
        match node_type {
            NodeType::Dir => {
                create_dir_all(target).await?;
                
                for t in &n.children {
                    q.push_back(t.clone());
                }
            },
            NodeType::File(_) => {
            }
        }
    }
    
    Ok(())
} */

pub async fn read_to_string(target: &Path) -> Result<String, Box<dyn Error + Sync + Send>> {
    let f = File::options().read(true).open(target).await?;             
    let mut reader = BufReader::new(f);
    let mut data = String::new();
    reader.read_to_string(&mut data).await.unwrap();
    Ok(data)
}

pub async fn read_filename(target: &Path) -> Result<String, Box<dyn Error + Sync + Send>> {
    let filename_ref = target.file_stem().ok_or("no file_stem")?.to_str().ok_or("file_stem to str failed")?;
    let filename = String::from(filename_ref); 
    Ok(filename)
}

pub async fn read_filename_with_ext(target: &Path) -> Result<String, Box<dyn Error + Sync + Send>> {
    let filename_ref = target.file_name().ok_or("no file_name")?.to_str().ok_or("file_name to str failed")?;
    let filename = String::from(filename_ref); 
    Ok(filename)
}


pub async fn write_from_string(target: &Path, s: String) -> Result<(), Box<dyn Error + Sync + Send>> {
    let f = File::options().write(true).create(true).open(&target).await?;
    let mut writer = BufWriter::new(f);
    writer.write(s.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}
pub async fn write_from_slice(target: &Path, b: &[u8]) -> Result<(), Box<dyn Error + Sync + Send>> {
    let f = File::options().write(true).create(true).truncate(true).open(&target).await?;
    let mut writer = BufWriter::new(f);
    writer.write(b).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn copy_recursive(from: &Path, to: &Path) -> Result<(), Box<dyn Error + Sync + Send>> {
    let mut q: VecDeque<(PathBuf, PathBuf)> = VecDeque::new();
    q.push_back((from.to_path_buf(), to.to_path_buf()));
    
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
