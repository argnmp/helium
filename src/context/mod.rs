use std::path::{Path, PathBuf};

use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub config: PathBuf, 
}

#[derive(Deserialize, Debug)]
pub struct Context {
    pub nodes: Vec<PathBuf>,
    pub target_base: PathBuf,
    pub open_file_limit: usize,
    pub render: RenderContext
}

#[derive(Deserialize, Debug)]
pub struct RenderContext {
    pub template: String,
    pub profile: Option<PathBuf>,
    pub r#static: Vec<PathBuf>,
    pub list_size: usize,
}

impl Context {
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml = std::fs::read_to_string(path)?;
        let context: Context = serde_yaml::from_str(&yaml)?;         
        
        Ok(context)
    }
}
