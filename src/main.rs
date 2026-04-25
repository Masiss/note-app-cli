use anyhow::Result;
use chrono::{DateTime, Local};
use clap::Parser;
use nanoid::nanoid;
use std::{path::Path, sync::LazyLock, time::SystemTime};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
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
    let args = Cli::parse();

    println!("{}: {}", args.action, args.content);
    match args.action.as_str() {
        "new" => write_note(&args.content).await?,
        "find" => find_note(&args.content).await?,
        _ => {
            println!("Cant find that action.")
        }
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
