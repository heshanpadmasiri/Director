import { invoke } from "@tauri-apps/api/tauri";
import { open } from '@tauri-apps/api/dialog';
import { appDir } from '@tauri-apps/api/path';

let selectedIndex = 0;
let maxIndex = 0;
let ignoreInput = false;

interface FileData {
    name: string;
    marked: boolean;
}

interface FilePreview {
    kind: string;
    content: string;
}

interface PreviewData {
    File?: FilePreview;
    Directory?: string;
}

async function updateFileList() {
    const dirElem = document.querySelector("#currentDir");
    const dir = await invoke<string>("get_current_path");
    console.log(dir);
    if (dirElem) {
        dirElem.textContent = dir;
    }
    const fileList = document.querySelector<HTMLElement>("#fileList");
    if (fileList) {
        removeChildren(fileList);
        const files = await invoke<FileData[]>("get_files");
        maxIndex = files.length - 1;
        files.forEach((fileData, index) => {
            const listItem = document.createElement("li");
            listItem.onclick = () => {
                onFileItemClick(index);
            };
            listItem.textContent = fileData.name;
            if (fileData.marked) {
                listItem.classList.add("marked");
            }
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
    if (ignoreInput) {
        return;
    }
    selectedIndex = index;
    // TODO: factor out preview from going to directory
    var previewData = await invoke<PreviewData>("get_preview", {index});
    const file = previewData['File'];
    if (file != null) {
        const preview = document.querySelector("#preview");
        switch (file.kind) {
            case 'Image':
                const base64 = file.content;
                if (preview) {
                    preview.innerHTML = `<img class="previewContent" src="data:image/jpeg;base64,${base64}">`;
                }
                break;
            default:
                if (preview) {
                    preview.innerHTML = `<p class="previewContent">${file.content}</p>`;
                }
                break;
        }
    }
    else {
        // TODO: handle directories
        await invoke("go_to_directory", {index});
        await updateFileList();
    }
    updateSelectedIndicator();
}

async function markFile() {
    // FIXME: marking directories doesn't work?
    await invoke("mark_file", {index: selectedIndex});
    // FIXME: this shouldn't move the cursor
    const fileList = document.querySelector("#fileList");
    const listItem = fileList?.querySelectorAll("li")[selectedIndex];
    if (listItem) {
        listItem.classList.toggle("marked");
    }
}

async function updatePreview(index: number) {
    var previewData = await invoke<PreviewData>("get_preview", {index});
    const file = previewData['File'];
    if (file != null) {
        const preview = document.querySelector("#preview");
        switch (file.kind) {
            case 'Image':
                const base64 = file.content;
                if (preview) {
                    preview.innerHTML = `<img class="previewContent" src="data:image/jpeg;base64,${base64}">`;
                }
                break;
            default:
                if (preview) {
                    preview.innerHTML = `<p class="previewContent">${file.content}</p>`;
                }
                break;
        }
    }
}

async function updateSelectedIndicator() {
    const fileList = document.querySelector("#fileList");
    const selected = fileList?.querySelector(".selected");
    if (selected) {
        selected.classList.remove("selected");
    }
    const listItem = fileList?.querySelectorAll("li")[selectedIndex];
    if (listItem) {
        listItem.classList.add("selected");
    }
}

async function onKeyPress(event: KeyboardEvent) {
    if (ignoreInput) {
        return;
    }
    switch (event.key) {
        case "j":
            if (selectedIndex < maxIndex) {
                selectedIndex++;
                await updateSelectedIndicator();
            }
            break;
        case "k":
            if (selectedIndex > 0) {
                selectedIndex--;
                await updateSelectedIndicator();
            }
            break;
        case "h":
            await goToParent();
            return;
        case "m":
            markFile();
            break;
        case "c":
            await copyFiles();
            break;
        case "l": // If it is file this will just show the preview so not a problem
        case "Enter":
            await onFileItemClick(selectedIndex);
            return;
    }
    await updatePreview(selectedIndex);
};

async function init() {
    const dirElem = document.querySelector<HTMLElement>("#currentDir");
    if (dirElem) {
        const dir = await invoke<string>("get_current_path");
        dirElem.textContent = dir;
        updateFileList();
        dirElem.onclick = async () => {
            await goToParent();
        }
    }
    document.addEventListener("keypress", onKeyPress);
}

async function goToParent() {
    await invoke("go_to_parent");
    await updateFileList();
}

async function copyFiles() {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: await appDir(),
    });
    if (selected) {
        ignoreInput = true;
        await invoke("copy_marked", {path: selected});
        ignoreInput = false;
    }
}


window.addEventListener("DOMContentLoaded", () => {
    init();
});
