// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::Read, path::PathBuf, sync::Mutex};

use tauri::State;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AppState {
    path: Mutex<PathBuf>,
    files: Mutex<Vec<PathBuf>>,
    marked_files: Mutex<Vec<PathBuf>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
enum FileKind {
    Image,
    Other,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FilePreview {
    kind: FileKind,
    content: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
enum PreviewData {
    File(FilePreview),
    Directory(String),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FileData {
    name: String,
    marked: bool,
}

fn initial_state() -> AppState {
    AppState {
        path: Mutex::new(starting_path()),
        files: Mutex::new(Vec::new()),
        marked_files: Mutex::new(Vec::new()),
    }
}

fn main() {
    tauri::Builder::default()
        .manage(initial_state())
        .invoke_handler(tauri::generate_handler![
            copy_marked,
            get_current_path,
            get_files,
            get_marked_files,
            get_marked_preview,
            get_preview,
            go_to_directory,
            go_to_parent,
            go_to_path,
            mark_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

enum File {
    File(PathBuf),
    Directory(PathBuf),
}

impl File {
    fn name(&self) -> String {
        match self {
            File::File(path) => path.file_name().unwrap().to_str().unwrap().to_string(),
            File::Directory(path) => path.file_name().unwrap().to_str().unwrap().to_string(),
        }
    }

    fn path(&self) -> PathBuf {
        match self {
            File::File(path) => path.to_path_buf(),
            File::Directory(path) => path.to_path_buf(),
        }
    }
}

impl From<PathBuf> for File {
    fn from(path: PathBuf) -> Self {
        if path.is_dir() {
            File::Directory(path)
        } else {
            File::File(path)
        }
    }
}

fn files_from_paths(paths: &[PathBuf]) -> Vec<File> {
    paths
        .iter()
        .map(|path| File::from(path.to_path_buf()))
        .collect()
}

#[tauri::command]
fn get_marked_files(state: State<AppState>) -> Vec<FileData> {
    let marked_files = state.marked_files.lock().unwrap();
    marked_files
        .iter()
        .map(|file| FileData {
            name: file.file_name().unwrap().to_str().unwrap().to_string(),
            marked: true,
        })
        .collect()
}

#[tauri::command]
fn get_files(state: State<AppState>) -> Vec<FileData> {
    let current_path = state.path.lock().unwrap();
    let marked_files = state.marked_files.lock().unwrap();
    get_files_in_directory(&current_path)
        .iter()
        .map(|file| {
            if marked_files.contains(&file.path()) {
                FileData {
                    name: file.name(),
                    marked: true,
                }
            } else {
                FileData {
                    name: file.name(),
                    marked: false,
                }
            }
        })
        .collect()
}

#[tauri::command]
fn get_preview(index: usize, state: State<AppState>) -> PreviewData {
    let current_path = state.path.lock().unwrap();
    get_file_preview(index, &get_files_in_directory(&current_path))
}

#[tauri::command]
fn get_marked_preview(index: usize, state: State<AppState>) -> PreviewData {
    let marked_files = state.marked_files.lock().unwrap();
    get_file_preview(index, &files_from_paths(&marked_files))
}

fn get_file_preview(index: usize, file_paths: &Vec<File>) -> PreviewData {
    match &file_paths[index] {
        File::File(path) => PreviewData::File(file_preview(path)),
        File::Directory(path) => PreviewData::Directory(path.to_str().unwrap().to_string()),
    }
}

#[tauri::command]
fn mark_file(index: usize, state: State<AppState>) {
    let mut marked_files = state.marked_files.lock().unwrap();
    let current_path = state.path.lock().unwrap();
    let files = get_files_in_directory(&current_path);
    match &files[index] {
        File::File(path) => {
            if marked_files.contains(path) {
                marked_files.retain(|file| file != path);
            } else {
                marked_files.push(path.to_path_buf());
            }
        }
        _ => {}
    }
}

#[tauri::command]
fn go_to_directory(index: usize, state: State<AppState>) {
    let mut current_path = state.path.lock().unwrap();
    let files = get_files_in_directory(&current_path);
    match &files[index] {
        File::Directory(path) => {
            *current_path = path.to_path_buf();
        }
        _ => {}
    }
}

#[tauri::command]
fn copy_marked(path: String, state: State<AppState>) {
    let path = PathBuf::from(path);
    let marked_files: Vec<PathBuf> = state.marked_files.lock().unwrap().to_vec();
    for file in marked_files {
        let dest = path.join(file.file_name().unwrap());
        println!("Copying {:?} to {:?}", file, dest);
        std::fs::copy(file, dest).unwrap();
    }
}

fn file_preview(path: &PathBuf) -> FilePreview {
    match file_kind(path) {
        FileKind::Image => FilePreview {
            kind: FileKind::Image,
            content: image_content(path),
        },
        FileKind::Other => FilePreview {
            kind: FileKind::Other,
            content: "NA".to_string(),
        },
    }
}

fn image_content(path: &PathBuf) -> String {
    let mut file = std::fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    base64::encode(buffer)
}

fn file_kind(path: &PathBuf) -> FileKind {
    match path.extension() {
        Some(extension) => match extension.to_str().unwrap() {
            "jpg" | "png" | "gif" => FileKind::Image,
            _ => FileKind::Other,
        },
        None => FileKind::Other,
    }
}

#[tauri::command]
fn go_to_parent(state: State<AppState>) {
    let mut path = state.path.lock().unwrap();
    let parent = path.parent().unwrap();
    *path = parent.to_path_buf();
}

#[tauri::command]
fn go_to_path(state: State<AppState>, path_str: String) {
    let mut path = state.path.lock().unwrap();
    *path = PathBuf::from(path_str);
}

#[tauri::command]
fn get_current_path(state: State<AppState>) -> String {
    state
        .path
        .lock()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn get_files_in_directory(path: &PathBuf) -> Vec<File> {
    std::fs::read_dir(path).unwrap().filter_map(|entry| {
        match entry {
            Err(_) => None,
            Ok(entry) => {
                let path = entry.path();
                if is_hidden_file(&path) {
                    None
                } else {
                    Some(path)
                }
            }
        }
    }).map(|path_buf| {
        if path_buf.is_dir() {
            File::Directory(path_buf)
        } else {
            File::File(path_buf)
        }
    }).collect()
}

fn is_hidden_file(path: &PathBuf) -> bool {
    path.file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with(".")
}

fn starting_path() -> PathBuf {
    let path = std::env::current_dir().unwrap();
    let path = path.to_str().unwrap();
    PathBuf::from(path)
}
