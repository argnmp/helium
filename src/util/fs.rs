use std::{error::Error, collections::VecDeque, path::{Path, PathBuf}};

use tokio::{fs::{copy, create_dir_all, read_dir, remove_file, File}, io::{AsyncWriteExt, BufWriter}, sync::OnceCell};
use walkdir::{DirEntry, WalkDir};

use crate::get_context;

use super::{resolve_osstr, sem::Lock};

struct FileLimit {
    lk: Lock
}
static FILE_LIMIT: OnceCell<FileLimit> = OnceCell::const_new();
async fn get_file_limit() -> Result<&'static FileLimit, Box<dyn std::error::Error + Send + Sync>> {
    let context = get_context().await;
    let limit = context.open_file_limit.checked_sub(20).ok_or("open file limit should be bigger than 20")?;

    let lk = Lock::new(&["file_desc_limit"]);
    lk.ready_size("file_desc_limit", limit).await?;
    Ok(FILE_LIMIT.get_or_init(|| async {
        FileLimit {
            lk,
        } 
    }).await)
}

pub async fn write_from_string(target: &Path, s: String) -> Result<(), Box<dyn Error + Sync + Send>> {
    /*
     * fs function must hold SemaphorePermit until it ends
     */
    let _sem = get_file_limit().await?.lk.access("file_desc_limit").await?;
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

pub async fn copy_file(from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /*
     * fs function must hold SemaphorePermit until it ends
     */
    let _sem = get_file_limit().await?.lk.access("file_desc_limit").await?;
    if from.is_dir() || to.is_dir() {
        return Err("not files".into());
    }
    copy(from, to).await?;
    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with('.'))
         .unwrap_or(false)
}

pub async fn remove_dir(path: &Path, remove_hidden: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { 
    /*
     * fs function must hold SemaphorePermit until it ends
     */
    let _sem = get_file_limit().await?.lk.access("file_desc_limit").await?;

    let entries = WalkDir::new(path);
    let entries = entries.into_iter().filter_entry(|e| { remove_hidden || !is_hidden(e) });
    for entry in entries {
        let entry = entry?;
        if entry.depth() == 1 {
            if entry.path().is_dir() {
                tokio::fs::remove_dir_all(entry.path()).await?;
            } else {
                remove_file(entry.path()).await?;
            }
        }
    }
    Ok(())
}

pub async fn copy_recursive(from: &Path, to: &Path, copy_dotfile: bool) -> Result<(), Box<dyn Error + Sync + Send>> {
    /*
     * fs function must hold SemaphorePermit until it ends
     */
    let _sem = get_file_limit().await?.lk.access("file_desc_limit").await?;

    let from: PathBuf = from.into();
    let mut to: PathBuf = to.into();
    if from.is_file() {
        to.push(resolve_osstr(from.file_name())?); 
    }

    let mut q: VecDeque<(PathBuf, PathBuf)> = VecDeque::new();
    q.push_back((from.to_path_buf(), to.to_path_buf()));
    
    while let Some((cursor, target)) = q.pop_front() {
        match cursor.is_dir() {
            true => {
                create_dir_all(&target).await?;
                let mut entries = read_dir(&cursor).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let mut tp = target.clone();
                    tp.push(entry.file_name());
                    q.push_back((entry.path(), tp));
                }
            },
            false => {
                if copy_dotfile || !cursor.file_name().ok_or("no file name")?.to_str().ok_or("cannot convert to str")?.starts_with('.') {
                    copy(&cursor, &target).await?;
                }
            }
        }
    }
    Ok(())
}

