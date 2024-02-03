import { invoke } from "@tauri-apps/api/tauri";
import { open } from '@tauri-apps/api/dialog';
import { appDir } from '@tauri-apps/api/path';

let selectedIndex = 0;
let maxIndex = 0;
let ignoreInput = false;
enum Mode {
    Normal,
    Marked,
    Search
}
let mode = Mode.Normal;

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

async function refreshFileList() {
    const dirElem = document.querySelector("#currentDir");
    const dir = await invoke<string>("get_current_path");
    if (dirElem) {
        dirElem.textContent = dir;
    }
    await updateFileList(await invoke<FileData[]>("get_files"));
}

async function switchToMarkedMode() {
    mode = Mode.Marked;
    console.log(await invoke<FileData[]>("get_marked_files"));
    await updateFileList(await invoke<FileData[]>("get_marked_files"));
}

async function updateFileList(files: FileData[]) {
    const fileList = document.querySelector<HTMLElement>("#fileList");
    if (fileList == null) {
        console.error("failed to find file list");
        return;
    }
    removeChildren(fileList);
    maxIndex = files.length - 1;
    files.forEach((fileData, index) => {
        const listItem = document.createElement("li");
        listItem.onclick = () => {
            updatePreview(index, true);
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
    updatePreview(0);
}

function removeChildren(elem: HTMLElement) {
    while (elem.firstChild) {
        elem.removeChild(elem.firstChild);
    }
}

async function fetchPreviewData(index: number): Promise<PreviewData> {
    let fn = mode == Mode.Marked ? "get_marked_preview" : "get_preview";
    return await invoke<PreviewData>(fn, { index });
}

async function markFile() {
    // FIXME: marking directories doesn't work?
    await invoke("mark_file", { index: selectedIndex });
    // FIXME: this shouldn't move the cursor
    const fileList = document.querySelector("#fileList");
    const listItem = fileList?.querySelectorAll("li")[selectedIndex];
    if (listItem) {
        listItem.classList.toggle("marked");
    }
}

async function updatePreview(index: number, clicked = false) {
    var previewData = await fetchPreviewData(index);
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
    else if (clicked) {
        await invoke("go_to_directory", { index });
        await refreshFileList();
    }
    if (clicked) {
        selectedIndex = index;
        updateSelectedIndicator();
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
        listItem.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
}

async function updateSearch(event: KeyboardEvent) {
    // 0. check if the key is Esc and if so reset the mode
    // 0.1 set regex "" to back end to reset backend as well
    // * if key is enter reset the mode but not the file list
    // 1. get the text from the search box
    console.log("search mode event", event)
    const searchBox = document.querySelector("#searchText")
    if (searchBox == null) {
        console.error("failed to find search box")
        return;
    }
    let _text = searchBox.textContent;
    let text = _text != null ? _text : "";
    switch (event.key) {
            case "Backspace":
                text = text.slice(0, text.length-1);
                break;
            case "Enter":
                mode = Mode.Normal;
                searchBox.classList.add("hidden")
                searchBox.textContent = "";
                return;
            case "Escape":
                text = "";
                mode = Mode.Normal;
                break;
            default:
                text += event.key;
                break;
    }
    searchBox.textContent = text;
    // 3. send the text to backend regex search
    await invoke("filter_files_by_regex", { regex: text })
    // 4. update the files
    await updateFileList(await invoke<FileData[]>("get_files"));
}

async function swithToSearchMode() {
    if (mode != Mode.Normal) {
        console.error("searching with in mode: "+ mode + " not implemented")
        return;
    }
    const searchBox = document.querySelector("#searchText")
    if (searchBox == null) {
        console.error("failed to find search box")
        return;
    }
    mode = Mode.Search;
    searchBox.classList.remove("hidden");
}

async function onKeyPress(event: KeyboardEvent) {
    if (ignoreInput) {
        return;
    }
    if (mode == Mode.Search) {
        return updateSearch(event);
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
        case "M":
            await switchToMarkedMode();
            break;
        case "o":
            await openDirectory();
            break;
        case "c":
            await copyFiles();
            break;
        case "/":
            swithToSearchMode();
            break;
        case "l": // If it is file this will just show the preview so not a problem
        case "Enter":
            await updatePreview(selectedIndex, true);
            return;
        case "Escape":
            mode = Mode.Normal;
            await invoke("filter_files_by_regex", { regex: "" });
            await refreshFileList();
            selectedIndex = 0;
            break;
    }
    await updatePreview(selectedIndex);
};

async function init() {
    const dirElem = document.querySelector<HTMLElement>("#currentDir");
    if (dirElem == null) {
        console.error("failed to find current dir");
        return;
    }
    const dir = await invoke<string>("get_current_path");
    dirElem.textContent = dir;
    refreshFileList();
    dirElem.onclick = async () => {
        await goToParent();
    }
    document.addEventListener("keypress", onKeyPress);
}

async function goToParent() {
    if (mode == Mode.Normal) {
        await invoke("go_to_parent");
    } else {
        mode = Mode.Normal;
    }
    await refreshFileList();
}

async function openDirectory() {
    const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: await appDir(),
    });
    if (selected) {
        console.log(selected);
        ignoreInput = true;
        await invoke("go_to_path", { pathStr: selected });
        ignoreInput = false;
        refreshFileList();
    }
}

async function copyFiles() {
    const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: await appDir(),
    });
    if (selected) {
        ignoreInput = true;
        await invoke("copy_marked", { path: selected });
        ignoreInput = false;
    }
}


window.addEventListener("DOMContentLoaded", () => {
    init();
});
