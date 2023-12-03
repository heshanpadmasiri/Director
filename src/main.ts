import { invoke } from "@tauri-apps/api/tauri";

let selectedIndex = 0;

async function updateFileList() {
    const dirElem = document.querySelector("#currentDir");
    const dir = await invoke("get_current_path");
    console.log(dir);
    if (dirElem) {
        dirElem.textContent = dir;
    }
    const fileList = document.querySelector("#fileList");
    if (fileList) {
        removeChildren(fileList);
        const files = await invoke("get_files");
        files.forEach((fileName, index) => {
            const listItem = document.createElement("li");
            listItem.onclick = () => {
                onFileItemClick(index);
            };
            listItem.textContent = fileName;
            fileList?.appendChild(listItem);
        });
        const firstItem = fileList.querySelector("li");
        if (firstItem) {
            firstItem.classList.add("selected");
        }
        onFileItemClick(0);
    }
    console.log(fileList);
}

function removeChildren(elem: HTMLElement) {
    while (elem.firstChild) {
        elem.removeChild(elem.firstChild);
    }
}

async function onFileItemClick(index: number) {
    var previewData = await invoke("get_preview", {index});
    if (previewData.hasOwnProperty("File")) {
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
        await invoke("go_to_directory", {index});
        await updateFileList();
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
            await goToParent();
        }
    }
}

async function goToParent() {
    await invoke("go_to_parent");
    await updateFileList();
}


window.addEventListener("DOMContentLoaded", () => {
    fileList = document.querySelector("#fileList");
    init();
});
