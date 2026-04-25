use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, Local};
use clap::Parser;
use dirs::{config_dir, home_dir};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
    time::SystemTime,
};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncWriteExt, BufReader},
    process::Command,
    sync::Mutex,
};
#[derive(Parser)]
struct Cli {
    action: String,
    content: String,
}

static NOTE_DIR: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::from("D:/notes/note/")));
#[tokio::main]
async fn main() -> Result<()> {
    init().await?;
    let args = Cli::parse();

    println!("{}: {}", args.action, args.content);
    match args.action.as_str() {
        "new" => write_note(&args.content).await?,
        "find" => find_note(&args.content).await?,
        "change" => change_dir(args.content).await?,
        _ => {
            println!("Cant find that action.")
        }
    }
    Ok(())
}
async fn change_dir(input: String) -> Result<()> {
    let config_file = config_dir().unwrap().join("note").join("config.txt");
    let new_dir = Path::new(&input);
    if !new_dir.exists() {
        return Err(anyhow!("new dir is not exist"));
    }
    let _ = fs::write(config_file, new_dir.to_str().unwrap()).await;
    Ok(())
}
fn default_dir() -> PathBuf {
    home_dir().unwrap().join("notes")
}
async fn init() -> Result<()> {
    let default_dir = default_dir().join("notes");
    if default_dir.exists() {
        let config = fs::read_to_string(&default_dir).await?;
        if Path::new(&config).exists() {
            return Ok(());
        }
        fs::create_dir_all(&config).await?;
        return Ok(());
    }
    let config_dir = config_dir()
        .ok_or_else(|| anyhow!("cant find config dir"))?
        .join("note")
        .join("config.txt");

    if config_dir.exists() {
        let _ = fs::write(config_dir.clone(), &default_dir.to_str().unwrap()).await;
    }
    Ok(())
}
async fn write_note(note: &str) -> Result<()> {
    let filename = nanoid!(10);
    let note_dir = NOTE_DIR.lock().await;
    let note_path = Path::new(&*note_dir).join(filename + ".txt");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(note_path)
        .await?;
    let _ = file.write_all(note.as_bytes()).await;
    Ok(())
}
async fn find_note(input: &str) -> Result<()> {
    let note_dir = NOTE_DIR.lock().await;
    let note = Command::new("rg")
        .arg("--files-with-matches")
        .arg(input)
        .current_dir(&*note_dir)
        .output()
        .await?;
    let output = String::from_utf8(note.stdout).unwrap();
    for filename in output.lines() {
        let note_path = Path::new(&*note_dir).join(filename);
        let created: SystemTime = tokio::fs::metadata(&note_path).await?.created()?;
        let datetime: DateTime<Local> = created.into();
        let content = fs::read(&note_path).await?;
        println!(
            "{}: {:?}",
            datetime.format("%Y-%m-%d %H:%M:%S"),
            String::from_utf8(content).unwrap()
        );
    }
    Ok(())
}
