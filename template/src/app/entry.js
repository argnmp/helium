if (
    localStorage.theme === "dark" ||
    (!("theme" in localStorage) &&
        window.matchMedia("(prefers-color-scheme: dark)").matches)
) {
    localStorage.theme = "dark";
    document.documentElement.classList.add("dark");
} else {
    localStorage.theme = "light";
    document.documentElement.classList.remove("dark");
}

// light: 1, dark: 2
/* function giscusColorToggle(type){
    let colorScheme = "transparent_dark";
    if(type==1)
        colorScheme = "light";
    else if(type==2)
        colorScheme = "dark";

    const iframe = document.querySelector('iframe.giscus-frame');
    if (!iframe) return;
    iframe.contentWindow.postMessage({ giscus: { setConfig: { theme: colorScheme}} }, 'https://giscus.app');  
} */

window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", (e) => {
        if (e.matches) {
            localStorage.theme = "dark";
            document.documentElement.classList.add("dark");
        } else {
            localStorage.theme = "light";
            document.documentElement.classList.remove("dark");
        }
    });

// this function, changes the colorscheme of giscus according to the colorscheme set by the user in localStorage and prefers-color-scheme when giscus iframe has loaded
/* function handleMessage(event) {
    if (event.origin !== "https://giscus.app") return;
    if (!(typeof event.data === "object" && event.data.giscus)) return;

    if (
        localStorage.theme === "dark" ||
        (!("theme" in localStorage) &&
            window.matchMedia("(prefers-color-scheme: dark)").matches)
    ) {
        giscusColorToggle(2);
    } else {
        giscusColorToggle(1);
    }

    // after refreshing the colorscheme, remove eventlistener
    window.removeEventListener('message', handleMessage);

}
window.addEventListener("message", handleMessage);
*/
