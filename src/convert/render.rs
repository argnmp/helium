use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tera::Tera;
use tokio::sync::OnceCell;

use crate::{get_context, util::fs::write_from_string};

use super::file::DocumentProperty;

pub struct Template {
    tera: Tera, 
    tera_context: tera::Context,
}
impl Template {
    fn get_context(&self) -> tera::Context {
        self.tera_context.clone() 
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Profile {
    page_title: Option<String>,
    name: Option<String>,
    image: Option<String>,
    descriptions: Option<Vec<String>>,
    links: Option<Vec<(String, String)>>,
    footer: Option<String>,
}
impl Profile {
    fn new() -> Self {
        let mut profile = Profile::default();
        profile.fill();
        profile
    }
    fn fill(&mut self) {
        if self.page_title.is_none() {
            self.page_title = Some("Blog".into());
        }
        if self.name.is_none() {
            self.name = Some("anonymous".into());
        }
        if self.descriptions.is_none() {
            self.descriptions = Some(Vec::new());
        }
        if self.links.is_none() {
            self.links = Some(Vec::new());
        }
    }
}


static TEMPLATE: OnceCell<Template> = OnceCell::const_new();
pub async fn get_template() -> &'static Template {
    TEMPLATE.get_or_init(|| async {
        let context = get_context().await;

        let mut tera = Tera::new(&context.render.template).unwrap();
        tera.autoescape_on(vec![]);
        let mut tera_context = tera::Context::new();

        if let Some(path) = &context.render.profile {
            let yaml = std::fs::read_to_string(path).unwrap();
            let mut profile: Profile = serde_yaml::from_str(&yaml).unwrap();         
            profile.fill();
            tera_context.insert("profile", &profile);
        } else {
            tera_context.insert("profile", &Profile::new());
        }

        Template {
            tera,
            tera_context
        }
    }).await
}


pub async fn create_post_page(target: &Path, markdown_html: &str, doc_property: &DocumentProperty) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let template = get_template().await; 

    let mut tera_context = template.get_context();
    tera_context.insert("title", &doc_property.title);
    tera_context.insert("aliases", &doc_property.aliases);
    tera_context.insert("author", &doc_property.author);
    tera_context.insert("created_at", &doc_property.created_at);
    tera_context.insert("tags", &doc_property.tags);
    tera_context.insert("post", &markdown_html);

    let commit = template.tera.render("post.html", &tera_context).unwrap();
    write_from_string(target, commit).await?;
    
    Ok(())
}

#[derive(Serialize)]
pub struct List {
    pub link: String,
    pub title: String,
    pub created_at: String,
    pub author: String,
    pub summary: String,
    pub child_node: usize,
    pub is_pinned: bool,
    pub cover_images: Vec<String>,
}

#[derive(Serialize)]
pub struct Page {
    pub index: i32,
    pub cursor: bool,
    pub href: String,
}

#[derive(Serialize)]
pub struct Prop {
    pub paged: bool,
    pub bottom_href: Option<String>,
    pub top_href: Option<String>,
}


pub async fn create_index_page(target: &Path, refresh: Option<PathBuf>, list: &Vec<List>, page_indices: &Vec<Page>, prop: &Prop) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let template = get_template().await; 
    
    let mut tera_context = template.get_context();
    tera_context.insert("refresh", &refresh);
    tera_context.insert("list", list);
    tera_context.insert("pages", page_indices);
    tera_context.insert("prop", prop);

    let commit = template.tera.render("list.html", &tera_context)?;
    write_from_string(target, commit).await?;
    Ok(()) 
}
