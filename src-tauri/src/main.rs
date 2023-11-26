// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_files, get_starting_path])
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

// TODO: take an optional path (current_dir)
// TODO: return type and file name
#[tauri::command]
fn get_files() -> Vec<String> {
    get_files_in_directory(get_path()).iter().map(|file| file.name()).collect()
}

#[tauri::command]
fn get_starting_path() -> String {
    get_path().file_name().unwrap().to_str().unwrap().to_string()
}

fn get_files_in_directory(path: PathBuf) -> Vec<File> {
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

fn get_path() -> PathBuf {
    let path = std::env::current_dir().unwrap();
    let path = path.to_str().unwrap();
    PathBuf::from(path)
}
