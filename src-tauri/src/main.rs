// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, io::Read, path::PathBuf, sync::Mutex};

use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
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
    None,
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
    let _ = fix_path_env::fix();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(get_home().join("director_log.log"))
        .unwrap();
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();
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
        .unwrap();
}

#[derive(Debug)]
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
    match get_preview_inner(index, state) {
        Ok(preview) => preview,
        Err(err) => {
            log::error!("failed to get preview {err}");
            PreviewData::Directory("NA".to_string())
        }
    }
}

fn get_preview_inner(index: usize, state: State<AppState>) -> Result<PreviewData, &'static str> {
    let current_path = state.path.lock().map_err(|_| "failed to lock app state")?;
    get_file_preview(index, &get_files_in_directory(&current_path))
}

#[tauri::command]
fn get_marked_preview(index: usize, state: State<AppState>) -> PreviewData {
    match get_marked_preview_inner(index, state) {
        Ok(preview) => preview,
        Err(err) => {
            log::error!("failed to get marked preview {err}");
            PreviewData::Directory("NA".to_string())
        }
    }
}

fn get_marked_preview_inner(
    index: usize,
    state: State<AppState>,
) -> Result<PreviewData, &'static str> {
    let marked_files = state
        .marked_files
        .lock()
        .map_err(|_| "failed to lock app state")?;
    get_file_preview(index, &files_from_paths(&marked_files))
}

fn get_file_preview(index: usize, file_paths: &Vec<File>) -> Result<PreviewData, &'static str> {
    log::info!("{index} {file_paths:?}");
    if file_paths.len() <= index {
        return Ok(PreviewData::None);
    }
    match &file_paths[index] {
        File::File(path) => Ok(PreviewData::File(file_preview(path))),
        File::Directory(path) => Ok(PreviewData::Directory(
            path.to_str()
                .ok_or("failed to parse directory path")?
                .to_string(),
        )),
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
    match get_current_path_inner(state) {
        Ok(path) => path,
        Err(err) => {
            log::error!("failed to get current path {err}");
            get_home().to_str().unwrap().to_owned()
        }
    }
}

fn get_current_path_inner(state: State<AppState>) -> Result<String, &'static str> {
    let path = state.path.lock().map_err(|_| "failed to lock app state")?;
    let path_str = if is_root_path(&path) {
        "/".to_string()
    } else {
        path.to_str()
            .ok_or("failed to convert path to string")?
            .to_string()
    };
    Ok(path_str)
}

fn is_root_path(path: &PathBuf) -> bool {
    path.parent().is_none()
}

fn get_files_in_directory(path: &PathBuf) -> Vec<File> {
    match get_files_in_directory_inner(path) {
        Ok(files) => files,
        Err(_) => {
            log::error!("failed to get files in directory");
            vec![]
        }
    }
}

fn get_files_in_directory_inner(path: &PathBuf) -> Result<Vec<File>, &'static str> {
    Ok(std::fs::read_dir(path)
        .map_err(|err| "failed to read dir")?
        .filter_map(|entry| match entry {
            Err(_) => None,
            Ok(entry) => {
                let path = entry.path();
                if is_hidden_file(&path) {
                    None
                } else {
                    Some(path)
                }
            }
        })
        .map(|path_buf| {
            if path_buf.is_dir() {
                File::Directory(path_buf)
            } else {
                File::File(path_buf)
            }
        })
        .collect())
}

fn is_hidden_file(path: &PathBuf) -> bool {
    path.file_name().unwrap().to_str().unwrap().starts_with(".")
}

fn starting_path() -> PathBuf {
    match std::env::current_dir() {
        Ok(path) => PathBuf::from(path.to_str().unwrap()),
        // FIXME: log this
        Err(_) => get_home(),
    }
}

fn get_home() -> PathBuf {
    match env::var("HOME") {
        Ok(home) => PathBuf::from(home),
        Err(_) => {
            panic!("Unable to determine user's home directory");
        }
    }
}
