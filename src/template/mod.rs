use std::{path::PathBuf, error::Error};
use tera::Tera;
pub mod raw;

pub struct Template {
    pub path: PathBuf,
    pub tera: Tera,
}
impl Template {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let tera = Tera::new(path.to_str().ok_or(crate::error::Error::new("template load failed"))?)?;
        Ok(Self {
            path,
            tera
        })
    }
}
