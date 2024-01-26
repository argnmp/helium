use std::{path::PathBuf, error::Error, collections::HashMap};
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all="lowercase")]
pub enum ResourceFlag {
    List,
    Layout,
    Element,
}
#[derive(Deserialize)]
pub struct Config {
    pub target: Vec<String>, 
    pub base: String,
    pub r#static: Vec<String>,
    pub template: String,
    pub resource: HashMap<ResourceFlag, String>,
}

pub struct Context {
    file: PathBuf,
    pub config: Config,
}

impl Context {
    pub fn new(file: PathBuf) -> Result<Self, Box<dyn Error>> {
        let yaml = std::fs::read_to_string(&file)?;
        let conf: Config = serde_yaml::from_str(&yaml)?;         
        Ok(Self {
            file: file.into(),
            config: conf,
        })
    }
}
