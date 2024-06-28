export default class Ground {
  constructor(){
    this.pages = new Map();
  }
  async add(url){
    let res = await fetch(url, {
      method: 'GET',
      mode: 'cors',
      headers: {
        "Accept": "text/html"
      }
    })

    let html = await res.text();
    let domParser = new DOMParser();
    let document = domParser.parseFromString(html, "text/html");
    
    this.pages.set(url, document);
  }
  async add_many(urls){
    let reqs = [];
    for(let url of urls) {
      reqs.push(fetch(url, {
        method: 'GET',
        mode: 'cors',
        headers: {
          "Accept": "text/html"
        }
      })); 
    }
    let ress = await Promise.all(reqs);
    
    let index = 0;
    for(let res of ress){
      let html = await res.text();
      let domParser = new DOMParser();
      let document = domParser.parseFromString(html, "text/html");
      this.pages.set(urls[index], document);
      index += 1;
    }
  }

  async add_included_anc(url) {
    let target = this.pages.get(url); 
    if(!target){
      console.log("requested page not cached");
      throw new Error();
    }
    let target_main = target.getElementById("main");
    let collection = target_main.getElementsByClassName("anc");
    let hrefs = [];
    for(let i = 0; i<collection.length; i++){
      let anc = collection.item(i);
      let href = anc.getAttribute("href");
      hrefs.push(href);
    }
    await this.add_many(hrefs);
  }

  async load(url) {
    let document = window.document;
    let main = document.getElementById("main");
    let target = this.pages.get(url);
    if(!target){
      console.log("requested page not cached");
      throw new Error();
    }
    let target_main = target.getElementById("main");
    let target_main_clone = target_main.cloneNode(true);
    main.replaceWith(target_main_clone);
  }
}
