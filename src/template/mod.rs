use std::{error::Error, path::PathBuf, str::from_utf8, collections::HashMap, pin::Pin};

use tokio::{fs::File, io::{BufReader, AsyncReadExt}};

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum TemplateFlag {
    List,
    Post,
    Element,
    ElementLink,
    Undefined,
}
impl From<&str> for TemplateFlag {
    fn from(value: &str) -> Self {
        match value {
            "list" => TemplateFlag::List,
            "post" => TemplateFlag::Post,
            "element" => TemplateFlag::Element,
            "elementlink" => TemplateFlag::ElementLink,
            _ => TemplateFlag::Undefined,
        }     
    }
}

#[derive(Debug)]
pub struct Template {
    buf: String,
    reference: Vec<(TemplateFlag, (usize, usize))>,
}

impl Template {
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

    pub async fn replace(&self, flag: TemplateFlag, target: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
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

#[tokio::test]
async fn regex_test(){
    let template = Template::new("./resource/index.html".into()).await.unwrap();
    assert!(true)
}
