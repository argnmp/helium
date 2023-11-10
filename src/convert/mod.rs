use std::{cell::RefCell, error::Error, rc::Rc, path::PathBuf, collections::{VecDeque, HashSet, hash_map::DefaultHasher}, sync::Arc, fmt::Debug};
use markdown::{Options, ParseOptions, mdast::{Text, InlineCode, Code}};
use serde::{Serialize, Deserialize};

use tokio::{fs::File, io::{BufWriter, AsyncWriteExt, BufReader, AsyncReadExt}};
use xorf::{HashProxy, Xor8};

use crate::{index::{Node, NodeProperty, NodeType, FileType}, TEMPLATE, TOKENIZER};

#[derive(Serialize)]
struct List {
    link: String,
    title: String,
    created_at: String,
    author: String,
    summarize: String,
}
impl List {
    fn new(link: &str, title: &str, created_at: &str, author: &str, summarize: &str) -> Self {
        List {
            link: link.to_owned(),
            title: title.to_owned(), 
            created_at: created_at.to_owned(),
            author: author.to_owned(),
            summarize: summarize.to_owned(),
        }
    }
}

pub async fn create_index_document(node: Rc<RefCell<Node>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let node = node.borrow();
    let NodeProperty {node_type, target, .. } = &node.property;
    match node_type {
        NodeType::Dir => {
            let mut list = Vec::new();
            for t in &node.children {
                let t = t.borrow_mut();
                let NodeProperty {node_type, rel, ..} = &t.property;
                
                match node_type {
                    NodeType::Dir => {
                        let mut rel2 = rel.clone();
                        rel2.pop();
                        rel2.push(format!("{}/index.html",rel.file_stem().unwrap().to_str().unwrap()));
                        list.push(List::new(rel2.to_str().unwrap(), rel.file_stem().unwrap().to_str().unwrap(), "", "", ""));
                    },
                    NodeType::File(FileType::Markdown(document)) => {
                        let Document { property, ..} = document;
                        let mut rel2 = rel.clone();
                        rel2.pop();
                        rel2.push(format!("{}.html",rel.file_stem().unwrap().to_str().unwrap()));
                        list.push(List::new(
                                rel2.to_str().unwrap(), 
                                rel2.file_stem().unwrap().to_str().unwrap(), 
                                &property.created_at.clone().unwrap_or("undefined".to_owned()),
                                &property.author.clone().unwrap_or("undefined".to_owned()),
                                &property.summarize.clone().unwrap_or("undefined".to_owned())
                                ));
                    },
                    _ => {}
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
        NodeType::File(_) => {
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
    pub summarize: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Document {
    pub property: DocumentProperty,
    pub raw: String,
    pub html: String,
    pub token: HashSet<String>,
}

impl Document {
    pub async fn new(path: PathBuf) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let f = File::options().read(true).open(&path).await?;             
        let mut reader = BufReader::new(f);
        let mut data = String::new();
        reader.read_to_string(&mut data).await.unwrap();
        
        let mut property = String::new();
        let mut raw = String::new();
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
                raw.push_str(line);
                raw.push_str("\n");
            }
            
        }
        let filename_ref = path.file_stem().unwrap().to_str().unwrap();
        let filename = String::from(filename_ref); 

        let mut property: DocumentProperty = serde_yaml::from_str(&property)?;
        property.title = Some(filename);

        let html = markdown::to_html_with_options(&raw, &Options::gfm())?;

        let mut token_before = Vec::new();
        let mut summarize = Vec::new();
        let mut summarize_size = 0;
        let mdast = markdown::to_mdast(&raw, &ParseOptions::gfm())?;
        let mut q = VecDeque::<&markdown::mdast::Node>::new();
        q.push_back(&mdast);
        while let Some(node) = q.pop_back() {
            match node {
                markdown::mdast::Node::Text(Text { value, .. }) => {
                    let values = value.split('\n');
                    for value in values {
                        // let mut t = value.split(|c: char| !c.is_alphabetic()).filter(|s| !s.is_empty()).map(|s: &str| s.to_string()).collect::<Vec<String>>();
                        // dbg!(&t);
                        // token_before.append(&mut t);
                        token_before.push(value.to_string());
                        if summarize_size < 300 {
                            summarize.push(tera::escape_html(value.trim()));
                            summarize_size += value.len();
                        }
                        else {
                            break;
                        }
                    }
                },
                markdown::mdast::Node::InlineCode(InlineCode { value, .. }) |
                markdown::mdast::Node::Code(Code { value, .. }) => {
                    let values = value.split('\n');
                    for value in values {
                        if summarize_size < 300 {
                            summarize.push(tera::escape_html(value.trim()));
                            summarize_size += value.len();
                        }
                        else {
                            break;
                        }
                    }
                },
                _ => {
                }
            } 
            match node.children() {
                Some(children) => {
                    for child in children.iter().rev() {
                        q.push_back(child);
                    }
                },
                None => {}
            }
        }
        let mut summ = String::new();
        for line in summarize {
            summ.push_str(&line);
            summ.push(' ');
        }
        property.summarize = Some(summ);

        let mut token = HashSet::new();
        for t in &token_before {
            let res = TOKENIZER.tokenize(&t).await?;
            res.data.into_iter().for_each(|s|{token.insert(s);});
        }

        Ok(Document { property, raw, html, token })
    }
}


#[derive(Deserialize, Serialize)]
pub struct SearchIndex {
    pub filter: HashProxy<String, DefaultHasher, Xor8>,
    pub title: String,
    pub rel: String,
}

pub fn create_search_index(node: Rc<RefCell<Node>>) -> Result<Vec<SearchIndex>, Box<dyn Error + Send + Sync>> {
    let node = node.borrow();
    let NodeProperty { node_type, source, target, rel } = &node.property;
    match node_type {
        NodeType::Dir => {
            let mut indices = Vec::new();
            for t in &node.children {
                let mut index = create_search_index(t.clone())?;
                indices.append(&mut index);
            }
            Ok(indices)
        },
        NodeType::File(FileType::Markdown(Document { property, raw, html, token })) => {
            let vec_tokens: Vec<String> = token.iter().map(|t|t.clone()).collect();
            let filter = HashProxy::from(&vec_tokens);

            let filename = rel.file_stem().unwrap().to_str().unwrap();
            let mut rel = rel.clone();
            rel.pop();
            rel.push(format!("{}.html",filename));
            
            Ok(vec![
               SearchIndex {
                   filter,
                   title: property.title.clone().ok_or("undefined")?,
                   rel: rel.to_str().ok_or("/")?.to_owned(),
               }
            ])
        },
        _ => {
            Ok(vec![])
        }
    }
}
