import { invoke } from "@tauri-apps/api/tauri";

let selectedIndex = 0;

async function updateFileList() {
    const fileList = document.querySelector("#fileList");
    if (fileList) {
        removeChildren(fileList);
        const files = await invoke("get_files");
        files.forEach((fileName, index) => {
            const listItem = document.createElement("li");
            listItem.onclick = () => {
                setPreview(index);
            };
            listItem.textContent = fileName;
            fileList?.appendChild(listItem);
        });
        const firstItem = fileList.querySelector("li");
        if (firstItem) {
            firstItem.classList.add("selected");
        }
        setPreview(0);
    }
    console.log(fileList);
}

function removeChildren(elem: HTMLElement) {
    while (elem.firstChild) {
        elem.removeChild(elem.firstChild);
    }
}

async function setPreview(index: number) {
    var previewData = await invoke("get_preview", {index});
    if (previewData.hasOwnProperty("File")) {
        console.log("file");
        previewData = previewData['File'];
        const preview = document.querySelector("#preview");
        switch (previewData['kind']) {
            case 'Image':
                const base64 = previewData['content'];
                if (preview) {
                    preview.innerHTML = `<img class="previewContent" src="data:image/jpeg;base64,${base64}">`;
                }
                break;
            default:
                if (preview) {
                    preview.innerHTML = `<p class="previewContent">${previewData['content']}</p>`;
                }
                break;
        }
    }
    else {
        // TODO: handle directories
        console.log(previewData);
    }
    const fileList = document.querySelector("#fileList");
    const selected = fileList?.querySelector(".selected");
    if (selected) {
        selected.classList.remove("selected");
    }
    const listItem = fileList?.querySelectorAll("li")[index];
    if (listItem) {
        listItem.classList.add("selected");
    }
}

async function init() {
    const dirElem = document.querySelector("#currentDir");
    if (dirElem) {
        const dir = await invoke("get_current_path");
        dirElem.textContent = dir;
        updateFileList();
        dirElem.onclick = async () => {
            await invoke("go_to_parent");
            const dir = await invoke("get_current_path");
            dirElem.textContent = dir;
            updateFileList();
        }
    }
}


window.addEventListener("DOMContentLoaded", () => {
    fileList = document.querySelector("#fileList");
    init();
});
