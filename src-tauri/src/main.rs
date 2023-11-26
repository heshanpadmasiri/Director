// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::Read, path::PathBuf, sync::Mutex};

use tauri::State;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct AppState {
    path: Mutex<PathBuf>,
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

fn initial_state() -> AppState {
    AppState {
        path: Mutex::new(starting_path()),
    }
}

fn main() {
    tauri::Builder::default()
        .manage(initial_state())
        .invoke_handler(tauri::generate_handler![
            get_files,
            get_current_path,
            go_to_parent,
            get_preview
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
}

#[tauri::command]
fn get_files(state: State<AppState>) -> Vec<String> {
    let current_path = state.path.lock().unwrap();
    get_files_in_directory(&current_path)
        .iter()
        .map(|file| file.name())
        .collect()
}

#[tauri::command]
fn get_preview(index: usize, state: State<AppState>) -> PreviewData {
    let current_path = state.path.lock().unwrap();
    let files = get_files_in_directory(&current_path);
    match &files[index] {
        File::File(path) => PreviewData::File(file_preview(path)),
        File::Directory(path) => PreviewData::Directory(path.to_str().unwrap().to_string()),
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
    let path = state.path.lock().unwrap().parent().unwrap().to_path_buf();
    *state.path.lock().unwrap() = path;
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
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.push(File::Directory(path));
        } else {
            files.push(File::File(path));
        }
    }
    files
}

fn starting_path() -> PathBuf {
    let path = std::env::current_dir().unwrap();
    let path = path.to_str().unwrap();
    PathBuf::from(path)
}
