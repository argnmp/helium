use std::collections::{HashMap, hash_map::DefaultHasher};

use js_sys::{Object, Uint8Array};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, RequestInit, RequestMode, Request, DomParser, Response, HtmlCollection, ReadableStreamDefaultReader};
use serde::{Deserialize};

use xorf::{HashProxy, Xor8, Filter, Xor16};

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
        // use promise_all
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

    pub async fn load(&mut self, url: String, do_transition: bool) -> Result<(), JsValue> {
        // web_sys::console::log_2(&"requested url:".into(), &url.clone().into());
        // self.print_cache().await?;
        let document = web_sys::window().ok_or("no window")?.document().ok_or("no document")?;
        let main = document.get_element_by_id("main").ok_or("current main id does not exist")?;

        let target = self.pages.get(&url).ok_or("requested page not cached")?;
        let target_main = target.get_element_by_id("main").ok_or("target main id does not exist")?;
        let target_main_clone = target_main.clone_node_with_deep(true)?;
        // fetch using promise all
        // find all anchor tag anc cache
        let collection = target_main.get_elements_by_class_name("anc");
        let mut hrefs = Vec::new();
        for i in 0..collection.length() {
            let anc = collection.item(i).ok_or("invalid anc index")?;
            let href = anc.get_attribute("href").ok_or("href attribute does not exist")?;
            // web_sys::console::log_1(&href.clone().into());
            hrefs.push(href);
        }
        // temporary fix: transition after fetching job
        if do_transition {
            main.replace_with_with_node_1(&target_main_clone)?;
        }

        self.add_many(hrefs).await?;
        Ok(())
    }

    pub async fn print_cache(&mut self) -> Result<(), JsValue> {
        self.pages.iter().for_each(|(k, _v)|{
            web_sys::console::log_1(&k.into());
        }); 
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Page {
    pub value: HashProxy<String, DefaultHasher, Xor16>,
    pub title: String,
    pub rel: String,
}
impl Page {
    pub fn decode(v: Vec<u8>) -> Self {
        bincode::deserialize(&v).unwrap()
    }
}

#[wasm_bindgen]
pub struct Index {
    pages: Vec<Page>,
}

#[wasm_bindgen]
impl Index {
    pub fn new() -> Self {
        Index {
            pages: Vec::new(),
        }
    }
    pub async fn load(&mut self, url: String) -> Result<(), JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts)?;
        request.headers()
            .set("Accept", "application/octet-stream")?;
        let window = web_sys::window().unwrap();
        let res = JsFuture::from(window.fetch_with_request(&request)).await?;
        let res: Response = res.dyn_into()?;
        let data = res.body().ok_or("body does not exist")?;
        let mut binary: Vec<u8> = Vec::new();
        let reader: ReadableStreamDefaultReader = data.get_reader().dyn_into()?;
        loop {
            let chunk = JsFuture::from(reader.read()).await?.dyn_into::<Object>()?;
            let done = js_sys::Reflect::get(&chunk, &"done".into())?;
            if done.is_truthy(){
                break;
            }
            let chunk = js_sys::Reflect::get(&chunk, &"value".into())?.dyn_into::<Uint8Array>()?;
            let binary_len = binary.len();
            binary.resize(binary_len + chunk.length() as usize, 255);
            chunk.copy_to(&mut binary[binary_len..]);
        }

        let pages: Vec<Page> = bincode::deserialize(&binary).map_err(|err|format!("debug: deserialize index failed: {:?}", err.to_string()))?;
        self.pages = pages;
        Ok(())
    }
    async fn attach(&self, result: Vec<(usize, String, String)>) -> Result<(), JsValue> {
        let mut target_links = String::new();
        for index in result {
            let target_link = format!(
                r"
                <a class='' href='{}'>
                <div class='mb-1 px-2 py-1 border border-gray-200 dark:border-gray-600 dark:text-gray-200'>
                {}
                </div> 
                </a>
                ", index.1, index.2);
            target_links.push_str(&target_link);
        }
        
        let target_wrapper = format!(
            r"
            <div class='' id='search_result'>
            {}
            </div>
            ", target_links);
        
        let dom = DomParser::new()?;
        let target = dom.parse_from_string(&target_wrapper, web_sys::SupportedType::TextHtml)?;
        let target_div = target.get_element_by_id("search_result").ok_or("search_result id does not exist in raw string")?;
        
        let document = web_sys::window().ok_or("no window")?.document().ok_or("no document")?;
        let main = document.get_element_by_id("search_result").ok_or("current search_result id does not exist")?;
        main.replace_with_with_node_1(&target_div)?;
        Ok(())
    }
    pub async fn search(&mut self, query: String) -> Result<(), JsValue> {
        let tokens = query.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s: &str| s.to_string())
            .collect::<Vec<String>>();

        let mut result: Vec<(usize, String, String)> = Vec::new();
        for page in &self.pages {
            let mut priority: usize = 0;
            for token in &tokens {
                if page.value.contains(&token) {
                    priority += 1;
                } else {
                    priority = 0;
                    break;
                }
            }
            if priority >= 1 {
                result.push((priority, page.rel.to_owned(), page.title.to_owned()));
            }
        }
        result.sort_by(|a, b| b.0.cmp(&a.0)); 
        self.attach(result).await?;
        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn start(){
}
