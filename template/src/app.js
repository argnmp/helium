import init, { Ground, Index } from "/static/render.js";

// external module should be re-called every page switch
async function load_external_modules(){

    // load highlight module    
    window.hljs.highlightAll();
    window.hljs.initLineNumbersOnLoad();
}
async function run() {
    await init();

    // load external module
    await load_external_modules();

    // load caching module
    let g = Ground.new();
    await g.add(window.location.pathname);
    await g.load(window.location.pathname, false);
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

    // load searching module
    let n = Index.new();
    await n.load("/static/searchindex");
    window.n = n;
    window.search = async function (e) {
        let text = document.getElementById(`search_input`).value;
        await n.search(text);
    };

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
run();
