use std::{cell::{RefMut, RefCell}, error::Error, rc::Rc};

use tokio::{fs::File, io::{BufWriter, AsyncWriteExt}};

use crate::{index::{Node, NodeProperty}, template::{TemplateFlag, Template}, CONTEXT, ctx::ResourceFlag};

pub async fn create_index_document(node: Rc<RefCell<Node>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let node = node.borrow();
    match &node.property {
        NodeProperty::Dir(_, target, _) => {
            // let list_template = Template::new(CONTEXT.config.resource.get(&ResourceFlag::Element).unwrap().into()).await?;
            let mut s = String::new();
            s.push_str("<ul>\n");
            for t in &node.children {
                let t = t.borrow_mut();
                match &t.property {
                    NodeProperty::Dir(_, _, relative) => {
                        s.push_str(&format!(r#"
<div class="py-5 px-2 border-b-2">
  <p class="text-2xl">
    <a href="{}">
      {} 
    </a>
  </p>
</div>
"#, relative.to_str().unwrap(), relative.file_stem().unwrap().to_str().unwrap()));
                    },
                    NodeProperty::File(_, _, relative) => {
                        s.push_str(&format!(r#"
<div class="py-5 px-2 border-b-2">
  <p class="text-2xl">
    <a href="{}">
      {} 
    </a>
  </p>
</div>
"#, relative.to_str().unwrap(), relative.file_stem().unwrap().to_str().unwrap()));
                    }
                }
            }
            s.push_str("</ul>\n");

            let mut target = target.clone();
            target.push("index.html");
            
            let template = Template::new(CONTEXT.config.resource.get(&ResourceFlag::Layout).unwrap().into()).await?;
            let commit = template.replace(TemplateFlag::List, &s).await?;
            dbg!(&target, &commit);
            let f = File::options().write(true).create(true).open(target).await?;
            let mut writer = BufWriter::new(f);
            writer.write(commit.as_bytes()).await?;
            writer.flush().await.unwrap();
        },
        NodeProperty::File(_, _, _) => {
        }
    }

    Ok(())
}
