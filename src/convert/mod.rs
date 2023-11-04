use std::{cell::{RefMut, RefCell}, error::Error, rc::Rc, collections::HashMap, path::PathBuf};
use serde::{Serialize, Deserialize};

use tokio::{fs::File, io::{BufWriter, AsyncWriteExt, BufReader, AsyncReadExt}};

use crate::{index::{Node, NodeProperty, NodeType}, CONTEXT, ctx::ResourceFlag, TEMPLATE};

#[derive(Serialize)]
struct List {
    link: String,
    title: String,
    created_at: String,
    author: String,
}
impl List {
    fn new(link: &str, title: &str, created_at: &str, author: &str) -> Self {
        List {
            link: link.to_owned(),
            title: title.to_owned(), 
            created_at: created_at.to_owned(),
            author: author.to_owned()
        }
    }
}

pub async fn create_index_document(node: Rc<RefCell<Node>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let node = node.borrow();
    let NodeProperty {node_type, source, target, rel, .. } = &node.property;
    match node_type {
        NodeType::Dir => {
            let mut list = Vec::new();
            for t in &node.children {
                let t = t.borrow_mut();
                let NodeProperty {node_type, rel, document, ..} = &t.property;
                
                match node_type {
                    NodeType::Dir => {
                        list.push(List::new(rel.to_str().unwrap(), rel.file_stem().unwrap().to_str().unwrap(), "", ""));
                    },
                    NodeType::File => {
                        match document {
                            Some(Document { property, .. }) => {
                                list.push(List::new(
                                        rel.to_str().unwrap(), 
                                        rel.file_stem().unwrap().to_str().unwrap(), 
                                        &property.created_at.clone().unwrap_or("undefined".to_owned()),
                                        &property.author.clone().unwrap_or("undefined".to_owned()),
                                        ));
                            },
                            None => {
                                list.push(List::new(
                                        rel.to_str().unwrap(), 
                                        rel.file_stem().unwrap().to_str().unwrap(), 
                                        "undefined",
                                        "undefined",
                                        ));

                            }
                        }
                    }
                }
            }

            let mut target = target.clone();
            target.push("index.html");
            
            let mut context = tera::Context::new();
            context.insert("list", &list);
            let commit = TEMPLATE.tera.render("list.html", &context)?;
            let f = File::options().write(true).create(true).open(target).await?;
            let mut writer = BufWriter::new(f);
            writer.write(commit.as_bytes()).await?;
            writer.flush().await.unwrap();
        },
        NodeType::File => {
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug, Clone)]
pub struct DocumentProperty {
    pub title: Option<String>,
    pub author: Option<String>,
    pub alias: Option<String>,
    pub created_at: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Document {
    pub property: DocumentProperty,
    pub body: String,
}
impl Document {
    pub async fn new(path: PathBuf) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let f = File::options().read(true).open(&path).await?;             
        let mut reader = BufReader::new(f);
        let mut data = String::new();
        reader.read_to_string(&mut data).await.unwrap();
        
        let mut property = String::new();
        let mut body = String::new();
        let mut property_on  = false;
        for (idx, line) in data.lines().enumerate() {
            if (idx == 0 && line == "---") || (property_on && line == "---") {
                property_on = !property_on;                
                continue;
            }
            if property_on {
                property.push_str(line);
                property.push_str("\n");
            }
            else {
                body.push_str(line);
                body.push_str("\n");
            }
            
        }
        let filename_ref = path.file_stem().unwrap().to_str().unwrap();
        let filename = String::from(filename_ref); 

        let mut property: DocumentProperty = serde_yaml::from_str(&property)?;
        property.title = Some(filename);

        Ok(Document { property, body })
    }
}
