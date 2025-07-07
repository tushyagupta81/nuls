use chrono::{DateTime, Utc};
use clap::Parser;
use nix::unistd::{Uid, User};
use owo_colors::OwoColorize;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};
use strum::Display;
use tabled::{
    settings::{
        object::{Columns, Rows},
        Alignment, Color, Style,
    },
    Table, Tabled,
};

#[derive(Debug, Display)]
enum FileType {
    File,
    Dir,
}

#[derive(Debug, Tabled)]
struct FileEntry {
    #[tabled{rename="Permissions"}]
    permissions: String,
    #[tabled{rename="Size"}]
    length: String,
    #[tabled{rename="Owner"}]
    owner: String,
    #[tabled{rename="Name"}]
    name: String,
    #[tabled{rename="Type"}]
    e_type: FileType,
    #[tabled{rename="Modified"}]
    modified: String,
}

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = "ls -la command in a table like format in mimicing the nu ls"
)]
struct Cli {
    path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(path_exists) = fs::exists(&path) {
        if path_exists {
            let files = get_files(&path);
            let mut table = Table::new(&files);
            table.with(Style::rounded());
            table.modify(Columns::last(), Color::FG_BLUE);
            table.modify(Columns::one(2), Color::FG_BRIGHT_YELLOW);
            table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
            table.modify(Rows::first(), Alignment::center());
            println!("{}", table);
        } else {
            println!("{}", "Path does not exist".red());
        }
    } else {
        println!("{}", "Error reading directory".red());
    }
}

fn get_files(path: &Path) -> Vec<FileEntry> {
    let mut data: Vec<FileEntry> = Vec::default();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir.flatten() {
            get_entries(entry, &mut data);
        }
    }

    data
}

fn get_entries(entry: fs::DirEntry, data: &mut Vec<FileEntry>) {
    if let Ok(meta_data) = fs::metadata(entry.path()) {
        data.push(FileEntry {
            permissions: permissions_to_string(&meta_data, meta_data.permissions().mode()),
            length: if meta_data.is_file() {
                parse_file_size(meta_data.len())
            } else {
                "-".cyan().to_string()
            },
            owner: uid_to_string(meta_data.uid()),
            name: parse_file_name(entry),
            e_type: if meta_data.is_dir() {
                FileType::Dir
            } else {
                FileType::File
            },
            modified: if let Ok(modi) = meta_data.modified() {
                let date: DateTime<Utc> = modi.into();
                format!("{}", date.format("%e %b %H:%M"))
            } else {
                String::default()
            },
        });
    }
}

fn parse_file_name(entry: fs::DirEntry) -> String {
    if entry.metadata().unwrap().is_dir() {
        (entry
            .file_name()
            .into_string()
            .unwrap_or("Unknown name".into()))
        .blue()
        .bold()
        .to_string()
    } else {
        (entry
            .file_name()
            .into_string()
            .unwrap_or("Unknown name".into()))
        .white()
        .to_string()
    }
}

fn parse_file_size(size: u64) -> String {
    if size < 1024 {
        size.to_string().green().to_string()
    } else if size > 1024 * 1024 {
        format!("{}m", (size as f64 / (1024.0 * 1024.0)).round())
            .bright_yellow()
            .to_string()
    } else {
        format!("{}k", (size as f64 / 1024.0).round())
            .bright_yellow()
            .to_string()
    }
}

fn uid_to_string(uid: u32) -> String {
    if let Ok(Some(user)) = User::from_uid(Uid::from(uid)) {
        user.name.to_string()
    } else {
        "User error".to_string()
    }
}

fn permissions_to_string(meta_data: &fs::Metadata, mode: u32) -> String {
    let mut result = String::new();
    let flags = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];

    if meta_data.is_dir() {
        result.push_str("d".bright_blue().to_string().as_str());
    } else {
        result.push_str(".".white().to_string().as_str());
    }

    for (i, (bit, ch)) in flags.iter().enumerate() {
        let colored = if mode & bit != 0 {
            if i < 3 {
                match ch {
                    'x' => ch.bright_yellow().to_string(),
                    'w' => ch.bright_red().to_string(),
                    'r' => ch.bright_yellow().to_string(),
                    _ => ch.to_string(),
                }
            } else {
                ch.green().to_string()
            }
        } else {
            "-".to_string().bright_black().to_string()
        };
        result.push_str(&colored);
        // Add space after each permission set (owner, group, others)
    }

    result.bold().to_string()
}
