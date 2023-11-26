import { invoke } from "@tauri-apps/api/tauri";

let fileList: HTMLElement | null;
let currentDir: string | null;

async function getFiles() {
    console.log("test1");
    if (fileList) {
        const files = await invoke("get_files");
        files.forEach(fileName => {
            const listItem = document.createElement("li");
            listItem.textContent = fileName;
            fileList?.appendChild(listItem);
        });
    }
    console.log(fileList);
}

async function init() {
    const dir = await invoke("get_starting_path");
    currentDir = dir;
    const dirElem = document.querySelector("#currentDir"); 
    if (dirElem) {
        dirElem.textContent = dir;
    }
}


window.addEventListener("DOMContentLoaded", () => {
    fileList = document.querySelector("#fileList");
    init();
    getFiles();
});
