use lazy_static::lazy_static;
use serde::Deserialize;
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

lazy_static! {
    static ref PATH: String = get_current_dir();
}

const CSS_FILE: &str = "style.css";

fn get_current_dir() -> String {
    match env::current_dir() {
        Ok(path) => path.to_string_lossy().into_owned(),
        Err(e) => {
            eprintln!("Error getting current directory: {}", e);
            String::new()
        }
    }
}

#[derive(Debug, Deserialize)]
struct PresentationInfo {
    name: String,
    path: String,
}

fn read_json<T: AsRef<Path>>(path: T) -> io::Result<PresentationInfo> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    serde_json::from_str(&content).map_err(|e| {
        eprintln!("Error deserializing JSON: {}", e);
        io::Error::new(io::ErrorKind::InvalidData, e)
    })
}

fn get_index_files() -> Vec<PresentationInfo> {
    let mut files = Vec::new();
    let target = PathBuf::from(PATH.clone()).join("slides");

    if let Ok(entries) = fs::read_dir(&target) {
        for entry in entries.flatten() {
            if entry.file_type().map_or(false, |t| t.is_dir()) {
                if let Ok(json_entry) = fs::read_dir(entry.path()) {
                    for file in json_entry.flatten() {
                        if let Some(ext) = file.path().extension() {
                            if ext == "json" {
                                if let Ok(info) = read_json(file.path()) {
                                    files.push(info);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    files
}

fn generate_main_index_file() -> String {
    let presentation_info = get_index_files();

    let list_items: String = presentation_info
        .iter()
        .map(|info| format!(r#"<a href="{}">{}</a> <br/>"#, info.path, info.name))
        .collect::<Vec<String>>()
        .join("\n");

    let html: String = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <link rel="stylesheet" href="{}">
                <title>My Presentations</title>
            </head>
            <body>
                <h1>My Presentations</h1>
                <div class="slide_container">
                    {}
                </div>
            </body>
        </html>
        "#,
        CSS_FILE, list_items
    );

    html
}

fn build_html() -> io::Result<()> {
    let html = generate_main_index_file();
    let path = PathBuf::from(&*PATH).join("index.html");

    let mut file = File::create(&path)?;
    file.write_all(html.as_bytes())?;

    println!("HTML file successfully created at: {}", path.display());
    Ok(())
}

fn main() -> io::Result<()> {
    build_html()
}

