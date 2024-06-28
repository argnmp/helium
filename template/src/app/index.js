import Ground from './module/ground.js';
let init;
let Index;

// external module should be re-called every page switch
async function load_external_modules(){
    // load highlight module    
    window.hljs.highlightAll();
    window.hljs.initLineNumbersOnLoad();
}
async function load_searching_module(){
    // load searching module
    let n = Index.new();
    let location = window.location.href.replace(/[^/]*$/, '');
    const loc = location.split('/');
    loc.pop();
    if(loc.length > 0 && !isNaN(Number(loc[loc.length-1]))){
        location = location.slice(0, location.length-1).replace(/[^/]*$/, '');
    }
        
    await n.load(location + "searchindex");
    window.n = n;
    window.search = async function (e) {
        let text = document.getElementById(`search_input`).value;
        await n.search(text);
    };

    // configure search button
    const searchToggle = document.getElementById("searchToggle");
    const searchModule = document.getElementById("searchModule");

    searchToggle.addEventListener("click", () => {
        searchModule.hidden = !searchModule.hidden;
    });


}

async function load_darkmode_module(){
    // load darkmode module
    const toggle = document.getElementById("toggle");
    toggle.addEventListener("click", () => {
        if (localStorage.theme === "dark") {
            localStorage.theme = "light";
            document.documentElement.classList.remove("dark");
        } else {
            localStorage.theme = "dark";
            document.documentElement.classList.add("dark");
        }
    });
}

async function load_toc_module(){
    const tocDataset = [];
    // register observer
    /*
    const observer = new IntersectionObserver(entries => {
        entries.forEach(entry => {
            let cur = tocDataset[Number(entry.target.dataset.index)]
            if(entry.isIntersecting && (-100 <= entry.boundingClientRect.y || entry.boundingClientRect.y <= 100)){
                cur.classList.remove('text-gray-900', 'dark:text-gray-200');
                cur.classList.add('text-customlight-700', 'dark:text-customdark-400', 'underline');
            } else {
                cur.classList.remove('text-customlight-700', 'dark:text-customdark-400', 'underline');
                cur.classList.add('text-gray-900', 'dark:text-gray-200');
            }
        })
    });
    */


    // create toc elements
    const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
    const toc = document.getElementById('toc');
    // reset toc
    while (toc.firstChild) {
        toc.removeChild(toc.lastChild);
    }

    const root = document.createElement('ul');
    const stk = [{layer: 0, elem: root}]; 
    const get_layer = (heading) => {
        if(heading.matches('h1')) return 1;
        else if(heading.matches('h2')) return 2;
        else if(heading.matches('h3')) return 3;
        else if(heading.matches('h4')) return 4;
        else if(heading.matches('h5')) return 5;
        else if(heading.matches('h6')) return 6;
    }
    let idx = 0;
    for(const heading of headings){
        // observe heading
        // observer.observe(heading);

        heading.classList.add('cursor-pointer', 'hover:text-customlight-700', 'dark:hover:text-customdark-400');
        heading.addEventListener('click', (e)=>{
            heading.scrollIntoView({ behavior: "smooth", block: "start", inline: "nearest" });
        })
        heading.dataset.index = idx;
        idx+=1
        
        let layer = get_layer(heading); 
        while(stk[stk.length-1].layer >= layer){
            stk.pop();
        }
        const sub_elem = document.createElement('li');
        sub_elem.classList.add('pl-4', 'pb-1');
        const p = document.createElement('p');
        p.classList.add('text-sm', 'text-gray-900', 'dark:text-gray-200', 'cursor-pointer', 'hover:text-customlight-700', 'dark:hover:text-customdark-400', 'hover:underline');
        p.innerText = heading.innerText; 
        p.addEventListener('click', (e)=>{
            heading.scrollIntoView({ behavior: "smooth", block: "start", inline: "nearest" });
        })
        tocDataset.push(p);
        const ul = document.createElement('ul');
        sub_elem.appendChild(p);
        sub_elem.appendChild(ul);
        stk[stk.length-1].elem.appendChild(sub_elem);
        stk.push({layer, elem: ul});
    }
    toc.appendChild(root);

}
async function load_cache_module(){
    const set_events = async () => {
        const ancs = document.getElementsByClassName('anc');
        for(const anc of ancs){
            anc.addEventListener("click",
                async function (e) {
                    e.preventDefault();
                    try {
                        const nextHref = anc.getAttribute("href");
                        history.pushState(null, null, nextHref);
                        await g.load(nextHref, true);
                        await g.add_included_anc(nextHref);
                        set_events();
                        await load_toc_module();
                        await load_searching_module();
                        await load_external_modules();
                    } catch (e) {
                        console.log(e);
                        window.location.href = anc.href;
                    }
                },
                false
            )
        }

    }
    // load caching module
    // let g = Ground.new();
    let g = new Ground();
    window.g = g;
    // await g.add(window.location.pathname);
    await g.add(decodeURI(window.location.pathname));
    await g.load(decodeURI(window.location.pathname));
    // because the main id element is re loaded, loading modules regarding to child elements of main should occur next.
    await g.add_included_anc(decodeURI(window.location.pathname));
    await load_external_modules();

    await set_events();
    
    window.onpopstate = async (e) => {
        e.preventDefault();
        try {
            await g.load(decodeURI(window.location.pathname), true);
            // await g.add_included_anc(decodeURI(window.location.pathname));
            await set_events();
            await load_toc_module();
            await load_searching_module();
            await load_external_modules();
        } catch (e) {
            console.log(e);
            window.location.href = window.location.pathname;
        }
    }
}

async function run() {

  // wasm is not bundled by webpack
  const module = await import(/* webpackIgnore: true */'/static/render.js');
  init = module.default;
  Index = module.Index;

  await init();
  await load_cache_module();
  await load_toc_module();
  await load_darkmode_module();
  await load_searching_module();
}

run();
