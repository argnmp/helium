(()=>{"use strict";class e{constructor(){this.pages=new Map}async add(e){let t=await fetch(e,{method:"GET",mode:"cors",headers:{Accept:"text/html"}}),a=await t.text(),n=(new DOMParser).parseFromString(a,"text/html");this.pages.set(e,n)}async add_many(e){let t=[];for(let a of e)t.push(fetch(a,{method:"GET",mode:"cors",headers:{Accept:"text/html"}}));let a=await Promise.all(t),n=0;for(let t of a){let a=await t.text(),o=(new DOMParser).parseFromString(a,"text/html");this.pages.set(e[n],o),n+=1}}async add_included_anc(e){let t=this.pages.get(e);if(!t)throw console.log("requested page not cached"),new Error;let a=t.getElementById("main").getElementsByClassName("anc"),n=[];for(let e=0;e<a.length;e++){let t=a.item(e).getAttribute("href");n.push(t)}await this.add_many(n)}async load(e){let t=window.document.getElementById("main"),a=this.pages.get(e);if(!a)throw console.log("requested page not cached"),new Error;let n=a.getElementById("main").cloneNode(!0);t.replaceWith(n)}}let t,a;async function n(){window.hljs.highlightAll(),window.hljs.initLineNumbersOnLoad()}async function o(){let e=a.new(),t=window.location.href.replace(/[^/]*$/,"");const n=t.split("/");n.pop(),n.length>0&&!isNaN(Number(n[n.length-1]))&&(t=t.slice(0,t.length-1).replace(/[^/]*$/,"")),await e.load(t+"searchindex"),window.n=e,window.search=async function(t){let a=document.getElementById("search_input").value;await e.search(a)};const o=document.getElementById("searchToggle"),c=document.getElementById("searchModule");o.addEventListener("click",(()=>{c.hidden=!c.hidden}))}async function c(){const e=[],t=document.querySelectorAll("h1, h2, h3, h4, h5, h6"),a=document.getElementById("toc");for(;a.firstChild;)a.removeChild(a.lastChild);const n=document.createElement("ul"),o=[{layer:0,elem:n}],c=e=>e.matches("h1")?1:e.matches("h2")?2:e.matches("h3")?3:e.matches("h4")?4:e.matches("h5")?5:e.matches("h6")?6:void 0;let l=0;for(const a of t){a.classList.add("cursor-pointer","hover:text-customlight-700","dark:hover:text-customdark-400"),a.addEventListener("click",(e=>{a.scrollIntoView({behavior:"smooth",block:"start",inline:"nearest"})})),a.dataset.index=l,l+=1;let t=c(a);for(;o[o.length-1].layer>=t;)o.pop();const n=document.createElement("li");n.classList.add("pl-4","pb-1");const i=document.createElement("p");i.classList.add("text-sm","text-gray-900","dark:text-gray-200","cursor-pointer","hover:text-customlight-700","dark:hover:text-customdark-400","hover:underline"),i.innerText=a.innerText,i.addEventListener("click",(e=>{a.scrollIntoView({behavior:"smooth",block:"start",inline:"nearest"})})),e.push(i);const d=document.createElement("ul");n.appendChild(i),n.appendChild(d),o[o.length-1].elem.appendChild(n),o.push({layer:t,elem:d})}a.appendChild(n)}!async function(){const l=await import("/static/render.js");t=l.default,a=l.Index,await t(),await async function(){const t=async()=>{const e=document.getElementsByClassName("anc");for(const l of e)l.addEventListener("click",(async function(e){e.preventDefault();try{const e=l.getAttribute("href");history.pushState(null,null,e),await a.load(e,!0),await a.add_included_anc(e),t(),await c(),await o(),await n()}catch(e){console.log(e),window.location.href=l.href}}),!1)};let a=new e;window.g=a,await a.add(decodeURI(window.location.pathname)),await a.load(decodeURI(window.location.pathname)),await a.add_included_anc(decodeURI(window.location.pathname)),await n(),await t(),window.onpopstate=async e=>{e.preventDefault();try{await a.load(decodeURI(window.location.pathname),!0),await t(),await c(),await o(),await n()}catch(e){console.log(e),window.location.href=window.location.pathname}}}(),await c(),await async function(){document.getElementById("toggle").addEventListener("click",(()=>{"dark"===localStorage.theme?(localStorage.theme="light",document.documentElement.classList.remove("dark")):(localStorage.theme="dark",document.documentElement.classList.add("dark"))}))}(),await o()}()})();