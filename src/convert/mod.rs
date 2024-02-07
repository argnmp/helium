use std::{error::Error, path::PathBuf, collections::{VecDeque, HashSet, hash_map::DefaultHasher, HashMap}, sync::Arc, fmt::Debug, borrow::Cow};
use markdown::{Options, ParseOptions, mdast::{Text, InlineCode, Code}};
use regex::{RegexBuilder, Captures};
use serde::{Serialize, Deserialize};

use tokio::{fs::File, io::{BufWriter, AsyncWriteExt, BufReader, AsyncReadExt}, sync::{Mutex, RwLock}};
use xorf::{HashProxy, Xor16};

use crate::{index::{Node, NodeProperty, NodeType, FileType, DirType}, TEMPLATE, TOKENIZER, fs};

#[derive(Serialize)]
struct List {
    link: String,
    title: String,
    created_at: String,
    author: String,
    summarize: String,
    child_node: usize,
}
impl List {
    fn new(link: &str, title: &str, created_at: &str, author: &str, summarize: &str, child_node: usize) -> Self {
        List {
            link: link.to_owned(),
            title: title.to_owned(), 
            created_at: created_at.to_owned(),
            author: author.to_owned(),
            summarize: summarize.to_owned(),
            child_node,
        }
    }
}

#[derive(Serialize)]
struct Page {
    index: usize,
    cursor: bool,
    href: String,
}

pub async fn create_index_document(node: Arc<RwLock<Node>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let node = node.read().await;
    let NodeProperty {node_type, target, rel, .. } = &node.property;
    let mut children = Vec::new();
    for node in &node.children {
        children.push(node.read().await);
    }
    children.sort_by(|a,b|{
        b.property.created.cmp(&a.property.created)  
    });
    
    match node_type {
        NodeType::Dir(dir_type) => {
            let mut list = Vec::new();
            for t in children {
                // let t = t.read().await;
                let NodeProperty {node_type, rel, ..} = &t.property;
                
                match node_type {
                    NodeType::Dir(DirType::Default(child_node_number, is_paged)) => {
                        let mut rel2 = rel.clone();
                        rel2.pop();
                        if *is_paged {
                            rel2.push(format!("{}/1/index.html",rel.file_name().unwrap().to_str().unwrap()));
                        } else {
                            rel2.push(format!("{}/index.html",rel.file_name().unwrap().to_str().unwrap()));
                        }
                        list.push(List::new(rel2.to_str().unwrap(), rel.file_name().unwrap().to_str().unwrap(), "", "", "", *child_node_number));
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
                                &property.summarize.clone().unwrap_or("undefined".to_owned()),
                                t.children.len()
                                ));
                    },
                    _ => {}
                }
            }


            let mut target = target.clone();
            target.push("index.html");
            
            let mut context = tera::Context::new();
            context.insert("html_title", "argnmp.github.io");
            context.insert("list", &list);
            
            let mut page_indices = Vec::new();
            if let DirType::Page(index, total) = dir_type {
                for i in 1..total+1 {
                    let mut rel = rel.clone();
                    rel.pop();
                    rel.push(&i.to_string());
                    page_indices.push(Page { index: i, cursor: i == *index, href: rel.to_str().ok_or("cannot convert path to str")?.to_owned()});
                }
            }
            context.insert("pages", &page_indices);

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

#[derive(Debug, Clone)]
pub struct Document {
    pub property: DocumentProperty,
    pub raw: String,
    pub html: Arc<Mutex<Option<String>>>,
    pub raw_token: HashSet<String>,
    pub token: Arc<Mutex<Option<HashSet<String>>>>,
    pub links: Vec<(usize, usize, DocumentLinkType)>,
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

        let mut raw_token = HashSet::new();
        let mut summarize = Vec::new();
        let mut summarize_size = 0;
        let mut links = Vec::new();
        let mdast = markdown::to_mdast(&raw, &ParseOptions::gfm())?;
        let mut q = VecDeque::<&markdown::mdast::Node>::new();
        q.push_back(&mdast);
        while let Some(node) = q.pop_back() {
            match node {
                markdown::mdast::Node::Text(Text { value, position }) => {
                    let values = value.split('\n');
                    for value in values {
                        raw_token.insert(value.to_string());
                        if summarize_size < 300 {
                            summarize.push(tera::escape_html(value.trim()));
                            summarize_size += value.len();
                        }
                        else {
                            break;
                        }
                    }
                    match position {
                        Some(position) => {
                            let p = position.start.offset;
                            let res = resolve_document_link(value)?;
                            let mut l = res.into_iter()
                                .map(|(start, end, link)|{
                                    (p + start, p + end, link)
                                })
                                .collect::<Vec<(usize, usize, DocumentLinkType)>>();
                            links.append(&mut l);
                        },
                        None => {

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

        Ok(Document { 
            property, 
            raw, 
            html: Arc::new(Mutex::new(None)), 
            raw_token,
            token: Arc::new(Mutex::new(None)),
            links,
        })
    }
    /* pub async fn resolve_document_link(&self, rels: Arc<HashMap<String, PathBuf>>) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut raw_document_link_resolved = (&self.raw).clone();
        let re = regex::Regex::new(r"\[\[([^\[\]]*)\]\]")?;
        for captures in re.captures_iter(&self.raw) {

            dbg!(&captures);
            let pos = captures.get(0).ok_or("no capture index 0")?;
            let name = captures.get(1).ok_or("no capture index 1")?;
            let target = rels.get(name.as_str());

            let index = pos.start().checked_sub(1).unwrap_or(pos.start());
            let prefix = if &self.raw[index..index+1] == "!" {"!"} else {""};
            match target {
                Some(path) => {
                    raw_document_link_resolved.replace_range(pos.start()..pos.end(), &format!("{}[{}]({})", prefix, name.as_str(), path.to_str().ok_or("failed to convert path to str")?));
                },
                None => {
                    raw_document_link_resolved.replace_range(pos.start()..pos.end(), &format!("{}[{}]({})", prefix, name.as_str(), "/"));
                }
            }
        }

        Ok(raw_document_link_resolved)
    } */
    pub async fn prepare_html(&self, included_files: Arc<HashMap<String, PathBuf>>) -> Result<&Self, Box<dyn Error + Send + Sync>> {
        let mut raw = self.raw.clone();
        let mut weight = 0;
        for (start, end, link_type) in &self.links {
            let start = start + weight;
            let end = end + weight;
            match link_type {
                DocumentLinkType::Document(name) => {
                    match included_files.get(name) {
                        Some(path) => {
                            let target = format!("[{}]({})", &name, path.to_str().ok_or("failed to convert path to str")?.replace(" ", "%20"));
                            raw.replace_range(start..end, &target);
                            weight += target.len() - (end - start);
                        },
                        None => {
                            let target = format!("[{}]({})", &name, "/");
                            raw.replace_range(start..end, &target);
                            weight += target.len() - (end - start);
                        },
                    }  
                },
                DocumentLinkType::Binary(name) => {
                    match included_files.get(name) {
                        Some(path) => {
                            let target = format!("![{}]({})", &name, path.to_str().ok_or("failed to convert path to str")?.replace(" ", "%20"));
                            raw.replace_range(start..end, &target);
                            weight += target.len() - (end - start);

                        },
                        None => {
                            let target = format!("![{}]({})", &name, "/");
                            raw.replace_range(start..end, &target);
                            weight += target.len() - (end - start);
                        }
                    }  

                }
            }
        }
        // dbg!(&raw);
        let html = markdown::to_html_with_options(&raw, &Options::gfm())?;
        
        let h2_regex = RegexBuilder::new(r"<h2>(.*?)<\/h2>").dot_matches_new_line(true).build()?;
        let result = h2_regex.replace_all(&html, |caps: &Captures| {
            format!("<h2 id=\"{}\">{}</h2>", &caps[1], &caps[1])
        });

        let mut html = Cow::Borrowed(&html);
        for i in 1..7 {
            let re = RegexBuilder::new(&format!(r"<h{}>(.*?)<\/h{}>", i, i)).dot_matches_new_line(true).build()?;
            let result = re.replace_all(&*html.to_owned(), |caps: &Captures| {
                format!("<h{} id=\"{}\">{}</h{}>",i, &caps[1], &caps[1], i)
            }).to_string();
            html = Cow::Owned(result);
        }

        *self.html.lock().await = Some(html.to_string());
        Ok(self)
    }
    pub async fn prepare_token(&self) -> Result<&Self, Box<dyn Error + Send + Sync>> {
        let stop_words = stop_words::get(stop_words::LANGUAGE::Korean);
        let mut token = HashSet::new();
        for t in &self.raw_token {
            let whitespace_token = t.split(|c: char| !c.is_alphabetic())
                .filter(|s| !s.is_empty())
                .filter(|s| !stop_words.contains(&s.to_string()))
                .map(|s: &str| s.to_string()).collect::<Vec<String>>();

            token.extend(whitespace_token.into_iter());

            let res = TOKENIZER.tokenize(&t).await?;
            token.extend(res.data.into_iter().filter(|token|{!stop_words.contains(&token)}));
        }
        if let Some(title) = &self.property.title {
            let title_token = title.split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
                .filter(|s| !stop_words.contains(&s.to_string()))
                .map(|s: &str| s.to_string()).collect::<Vec<String>>();
            token.extend(title_token.into_iter());
            
            let res = TOKENIZER.tokenize(&title).await?;
            token.extend(res.data.into_iter().filter(|token|{!stop_words.contains(&token)}));
        }
        // dbg!(&token);
        *self.token.lock().await = Some(token);

        Ok(self)
    }
}
#[derive(Debug, Clone)]
pub enum DocumentLinkType {
    Document(String),
    Binary(String),
}
pub fn resolve_document_link(s: &str) -> Result<Vec<(usize, usize, DocumentLinkType)>, Box<dyn Error + Send + Sync>> {
    let mut res = Vec::new();
    let re = regex::Regex::new(r"\[\[([^\[\]]*)\]\]")?;
    for captures in re.captures_iter(s) {
        //dbg!(&captures);
        let pos = captures.get(0).ok_or("no capture index 0")?;
        let name = captures.get(1).ok_or("no capture index 1")?;

        let index = pos.start().checked_sub(1).unwrap_or(pos.start());
        // let prefix = if &self.raw[index..index+1] == "!" {"!"} else {""};
        match &s[index..index+1] {
            "!" => {
                res.push((index,pos.end(), DocumentLinkType::Binary(name.as_str().to_owned())));
            },
            _ => {
                res.push((pos.start(),pos.end(), DocumentLinkType::Document(name.as_str().to_owned())));
            }
        }
    }
    Ok(res)
}


#[derive(Deserialize, Serialize)]
pub struct SearchIndex {
    pub filter: HashProxy<String, DefaultHasher, Xor16>,
    pub title: String,
    pub rel: String,
}

pub async fn create_search_index(node: Arc<RwLock<Node>>) -> Result<SearchIndex, Box<dyn Error + Send + Sync>> {
    let node = node.read().await;
    let NodeProperty { node_type, rel, .. } = &node.property;
    match node_type {
        NodeType::File(FileType::Markdown(Document { property, token, .. })) => {
            let Some(ref token) = *token.lock().await else { return Err("token is not extracted from raw token".into()); };
            // dbg!(&token);
            let vec_tokens: Vec<String> = token.iter().map(|t|t.clone()).collect();
            let filter = HashProxy::from(&vec_tokens);

            let filename = fs::read_filename(&rel).await?;
            let mut rel = rel.clone();
            rel.pop();
            rel.push(format!("{}.html",filename));
            
            Ok(SearchIndex {
                   filter,
                   title: property.title.clone().ok_or("undefined")?,
                   rel: rel.to_str().ok_or("/")?.to_owned(),
               }
            )
        },
        _ => {
            Err("no document file type".into())
        }
    }

}
