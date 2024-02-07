import init, { Ground, Index } from "/static/render.js";

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


    // create toc elements
    const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
    const toc = document.getElementById('toc');
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
        observer.observe(heading);

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

async function run() {
    await init();

    // load external module
    await load_external_modules();

    // load caching module
    let g = Ground.new();
    await g.add(window.location.pathname);
    await g.load(window.location.pathname, false);
    /*
    document.querySelector("html").addEventListener(
        "click",
        async function (e) {
            e.preventDefault();
            var anchor = e.target.closest("a");
            if (anchor !== null) {
                try {
                    const nextHref = anchor.getAttribute("href");
                    await g.load(nextHref, true);
                    history.pushState(null, null, nextHref);
                    await load_external_modules();
                } catch (e) {
                    window.location.href = anchor.href;
                }
            }
        },
        false
    );
    window.onpopstate = async (e) => {
        e.preventDefault();
        try {
            await g.load(location.pathname, true);
            load_external_modules();
        } catch (e) {
            window.location.href = location.pathname;
        }
    }
    */

    load_toc_module();
    load_darkmode_module();
    load_searching_module();
}
run();
