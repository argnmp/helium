use std::collections::HashMap;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, RequestInit, RequestMode, Request, DomParser, Response, HtmlCollection};

#[wasm_bindgen]
pub struct Ground{
    pages: HashMap<String, Document>,
}

#[wasm_bindgen]
impl Ground{
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
        }
    }
    pub async fn add(&mut self, url: String) -> Result<(), JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts)?;
        request.headers()
            .set("Accept", "text/html")?;

        let window = web_sys::window().unwrap();
        let res = JsFuture::from(window.fetch_with_request(&request)).await?;
        let res: Response = res.dyn_into()?;
        let html = JsFuture::from(res.text()?).await?.as_string().ok_or("failed to convert response body to str")?;
        let dom = DomParser::new()?;
        let document = dom.parse_from_string(&html, web_sys::SupportedType::TextHtml)?;

        self.pages.insert(url, document.clone());
        
        Ok(())
    }
    pub async fn add_many(&mut self, urls: Vec<String>) -> Result<(), JsValue> {
        let requests = js_sys::Array::new();
        for url in &urls {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            let request = Request::new_with_str_and_init(&url, &opts)?;
            request.headers()
                .set("Accept", "text/html")?;
            let window = web_sys::window().unwrap();
            requests.push(&window.fetch_with_request(&request));
        }
        let res = JsFuture::from(js_sys::Promise::all(&requests)).await?;

        let htmls = js_sys::Array::new();
        let iterable = js_sys::try_iter(&res)?.ok_or("not a valid iterable js object")?;
        for res in iterable {
            let res: Response = res?.dyn_into()?;
            htmls.push(&(res.text()?.into())); 
        }
        let res = JsFuture::from(js_sys::Promise::all(&htmls)).await?;
        
        for (url, html) in urls.into_iter().zip(js_sys::try_iter(&res)?.ok_or("not a valid iterable js object")?) {
            let dom = DomParser::new()?;
            let html = html?.as_string().ok_or("failed to convert response body to str")?;
            let document = dom.parse_from_string(&html, web_sys::SupportedType::TextHtml)?;

            self.pages.insert(url, document.clone());
        }
        
        Ok(())
    }

    pub async fn load(&mut self, url: String) -> Result<(), JsValue> {
        let document = web_sys::window().ok_or("no window")?.document().ok_or("no document")?;
        let main = document.get_element_by_id("main").ok_or("current main id does not exist")?;

        let target = self.pages.get(&url).ok_or("requested page not cached")?;
        let target_main = target.get_element_by_id("main").ok_or("target main id does not exist")?;

        main.replace_with_with_node_1(&target_main)?;
        
        /* fetch one by one
        
        // find all anchor tag anc cache
        let collection = target_main.get_elements_by_class_name("anc");
        // let mut hrefs = Vec::new();
        for i in 0..collection.length() {
            let anc = collection.item(i).ok_or("invalid anc index")?;
            let href = anc.get_attribute("href").ok_or("href attribute does not exist")?;
            self.add(href).await?;
            // hrefs.push(href);
        }
        // self.add_many(hrefs).await?; */
        
        
        // fetch using promise all
        // find all anchor tag anc cache
        let collection = target_main.get_elements_by_class_name("anc");
        let mut hrefs = Vec::new();
        for i in 0..collection.length() {
            let anc = collection.item(i).ok_or("invalid anc index")?;
            let href = anc.get_attribute("href").ok_or("href attribute does not exist")?;
            hrefs.push(href);
        }
        self.add_many(hrefs).await?;
        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn start(){
}
