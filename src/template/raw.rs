use std::{error::Error, path::PathBuf};

use tokio::{fs::File, io::{BufReader, AsyncReadExt}};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum RawTemplateFlag {
    List,
    Post,
    Element,
    ElementLink,
    Undefined,
}
impl From<&str> for RawTemplateFlag {
    fn from(value: &str) -> Self {
        match value {
            "list" => RawTemplateFlag::List,
            "post" => RawTemplateFlag::Post,
            "element" => RawTemplateFlag::Element,
            "elementlink" => RawTemplateFlag::ElementLink,
            _ => RawTemplateFlag::Undefined,
        }     
    }
}

#[derive(Debug)]
pub struct RawTemplate {
    buf: String,
    reference: Vec<(RawTemplateFlag, (usize, usize))>,
}

impl RawTemplate {
    pub async fn new(path: PathBuf) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let f = File::options().read(true).open(path).await?;
        let mut reader = BufReader::new(f);

        let mut buf = String::new();
        let mut reference = Vec::new(); 
        reader.read_to_string(&mut buf).await?;
        
        let re = regex::Regex::new(r"\[\[([^\[\]]*)\]\]").unwrap();
        for captures in re.captures_iter(&buf) {
            let pos = captures.get(0).unwrap();
            let name = captures.get(1).unwrap();
            reference.push((name.as_str().into(), (pos.start(), pos.end())));
        }
        
        Ok(Self {
            buf,
            reference,
        })
    }

    pub async fn replace(&self, flag: RawTemplateFlag, target: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut res = String::new();
        let mut cursor: usize = 0;
        for (fl, pos) in &self.reference {
            res.push_str(&self.buf[cursor..pos.0]);
            if fl == &flag {
                res.push_str(target);
            }
            cursor = pos.1 + 1;
        }
        res.push_str(&self.buf[cursor..(&self.buf).len()]);
        Ok(res)
    }
}
