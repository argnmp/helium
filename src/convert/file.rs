use std::{borrow::Cow, collections::{HashMap, HashSet, VecDeque}, path::{Path, PathBuf}, sync::Arc};

use markdown::{mdast::{Code, Image, InlineCode, Text}, Options, ParseOptions};
use regex::{Captures, RegexBuilder};
use serde::Deserialize;
use tokio::{fs::read_to_string, sync::RwLock};

use crate::{index::{Node}, util::{resolve_osstr, resolve_path, sem::Lock, token::get_tokenizer}};

#[derive(Debug)]
pub enum FileType {
    Markdown(FileProperty, Document),
    Binary(FileProperty),
}
impl FileType {
    pub async fn new(node: &Arc<Node>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path = &node.path.read().await.org_path;
        match resolve_osstr(path.extension())? {
            "md" => {
                let file_stem = resolve_osstr(path.file_stem())?;
                Ok(FileType::Markdown(FileProperty::new(file_stem.into())?, Document::from_path(path).await?))
            },
            _ => {
                Ok(FileType::Binary(FileProperty::from_path(path)?))
            }
        }
    }
}

#[derive(Debug)]
pub struct FileProperty {
    pub key: String,
}
impl FileProperty {
    fn new(key: String) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(FileProperty { key,})
    }
    fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let file_name = resolve_osstr(path.file_name())?;

        Ok(FileProperty { key: file_name.into() })
    }
}

#[derive(Debug)]
pub struct Document {
    pub raw: String,
    pub property: DocumentProperty,
    pub parameter: DocumentParameter,
    pub html: Option<String>,
}
#[derive(Deserialize, Debug, Clone)]
pub struct DocumentProperty {
    pub title: Option<String>,
    pub author: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub created_at: Option<String>,
    pub tags: Option<Vec<String>>,
    pub priority: Option<usize>,
}
#[derive(Debug)]
pub struct DocumentParameter {
    pub token: HashSet<String>,
    pub summary: String,
    pub link: Vec<(usize, usize, DocumentLinkType)>,
    pub image: RwLock<Vec<String>>,
    pub image_lk: Lock, 
}

#[derive(Debug, Clone)]
pub enum DocumentLinkType {
    Resource(String),
    Image(String),
}

impl Document {
    pub async fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let data = read_to_string(path).await?;

        /*
         * parse document property
         */
        let (mut property, raw) = parse_property(&data).await?;
        let title: String = resolve_osstr(path.file_name())?.into();
        property.title = Some(title.clone());

        /*
         * parse document parameters
         */
        let (raw_token, summary, link, image) = parse_parameter(&raw).await?;
        let token = create_token(raw_token, title).await?;
        let parameter = DocumentParameter {
            token,
            summary,
            link,
            image: RwLock::new(image),
            image_lk: Lock::new(&["image"])
        };

        Ok(Document {
            raw,
            property,
            parameter,
            html: None,
        })
    }
}

async fn parse_property(data: &str) -> Result<(DocumentProperty, String), Box<dyn std::error::Error + Send + Sync>> {
    let mut raw = String::new();
    let mut property = String::new();
    let mut flag = false;
    for (idx, line) in data.lines().enumerate() {
        if (idx == 0 && line == "---") || (flag && line == "---") {
            flag = !flag;                
            continue;
        }
        if flag {
            property.push_str(line);
            property.push('\n');
        }
        else {
            raw.push_str(line);
            raw.push('\n');
        }

    }

    let property: DocumentProperty = serde_yaml::from_str(&property)?;
    Ok((property, raw))
}
pub fn parse_document_link(s: &str) -> Result<Vec<(usize, usize, DocumentLinkType)>, Box<dyn std::error::Error + Send + Sync>> {
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
                res.push((index,pos.end(), DocumentLinkType::Image(name.as_str().to_owned())));
            },
            _ => {
                res.push((pos.start(),pos.end(), DocumentLinkType::Resource(name.as_str().to_owned())));
            }
        }
    }
    Ok(res)
}
async fn parse_parameter(data: &str) -> Result<(HashSet<String>, String, Vec<(usize, usize, DocumentLinkType)>, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    let mut raw_token = HashSet::new();
    let mut summarize = Vec::new();
    let mut summarize_size = 0;
    let mut link = Vec::new();
    let mut image = Vec::new();

    let mdast = markdown::to_mdast(data, &ParseOptions::gfm()).map_err(|_|{"markdown AST build failed"})?;
    let mut q: VecDeque<&markdown::mdast::Node> = VecDeque::new();
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
                        let res = parse_document_link(value)?;
                        let mut l = res.into_iter()
                            .map(|(start, end, link)|{
                                (p + start, p + end, link)
                            })
                        .collect::<Vec<(usize, usize, DocumentLinkType)>>();
                        link.append(&mut l);
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
                markdown::mdast::Node::Image(Image { url, .. }) => {
                    image.push(url.clone());
                }
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
    let mut summary = String::new();
    for line in summarize {
        summary.push_str(&line);
        summary.push(' ');
    }
    Ok((raw_token, summary, link, image))

}
pub async fn create_token(raw_token: HashSet<String>, title: String) -> Result<HashSet<String>, Box<dyn std::error::Error + Send + Sync>> {
    let tokenizer = get_tokenizer().await?;
    let mut token = HashSet::new();

    let stop_words = stop_words::get(stop_words::LANGUAGE::Korean);
    for t in raw_token {
        /*
         * todo: tokens from spliting whitespaces ?
         */

        let _whitespace_token = t.split(|c: char| !c.is_alphabetic())
            .filter(|s| !s.is_empty())
            .filter(|s| !stop_words.contains(&s.to_string()))
            .map(|s: &str| s.to_string()).collect::<Vec<String>>();

        // token.extend(whitespace_token.into_iter());

        let res = tokenizer.tokenize(&t).await?;
        // println!("from: {:?}, to: {:?}", &t, &res);
        token.extend(res.data.into_iter().filter(|token|{!stop_words.contains(token)}));
    }

    let title_token = title.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .filter(|s| !stop_words.contains(&s.to_string()))
        .map(|s: &str| s.to_string()).collect::<Vec<String>>();
    token.extend(title_token.into_iter());

    let res = tokenizer.tokenize(&title).await?;
    token.extend(res.data.into_iter().filter(|token|{!stop_words.contains(token)}));

    Ok(token)
}

pub async fn convert_html(doc: &Document, resource_map: Arc<HashMap<String, PathBuf>>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut raw = doc.raw.clone();
    let mut image = doc.parameter.image.write().await;
    let mut weight = 0;
    for (start, end, link_type) in &doc.parameter.link {
        let start = start + weight;
        let end = end + weight;
        match link_type {
            DocumentLinkType::Resource(name) => {
                match resource_map.get(name) {
                    Some(path) => {
                        let target = format!("[{}]({})", &name, resolve_path(path)?.replace(' ', "%20"));
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
            DocumentLinkType::Image(name) => {
                match resource_map.get(name) {
                    Some(path) => {
                        let target = format!("![{}]({})", &name, resolve_path(path)?.replace(' ', "%20"));
                        raw.replace_range(start..end, &target);
                        weight += target.len() - (end - start);

                        /*
                         * local images have higher priority than outside images
                         */
                        image.insert(0, resolve_path(path)?.into());
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
    doc.parameter.image_lk.ready("image").await?;

    let html = markdown::to_html_with_options(&raw, &Options::gfm()).map_err(|_|{"markdown to html failed"})?;

    let mut html = Cow::Borrowed(&html);
    for i in 1..7 {
        let re = RegexBuilder::new(&format!(r"<h{}>(.*?)<\/h{}>", i, i)).dot_matches_new_line(true).build()?;
        let result = re.replace_all(&html.to_owned(), |caps: &Captures| {
            format!("<h{} id=\"{}\">{}</h{}>",i, &caps[1], &caps[1], i)
        }).to_string();
        html = Cow::Owned(result);
    }

    Ok(html.to_string())
}
