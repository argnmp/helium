import init, { Ground, Index } from "/static/render.js";
async function run() {
  await init();
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
          await g.load(anchor.href, true);
        } catch (e) {
          window.location.href = anchor.href;
        }
      }
    },
    false
  );
  let n = Index.new();
  await n.load("/static/searchindex");
  window.n = n;
  window.search = async function (e) {
    let text = document.getElementById(`search_input`).value;
    await n.search(text);
  };
  window.hljs.highlightAll();
  window.hljs.initLineNumbersOnLoad();

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
