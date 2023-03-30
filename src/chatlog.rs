use anyhow::Context;
use colored::Colorize;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::consts::PROJECT_NAME;

#[derive(Serialize, Deserialize, Debug)]
pub enum Author {
    User,
    Bot,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogMessage {
    pub author: Author,
    pub text: String,
}

impl LogMessage {
    pub fn bot(text: impl Into<String>) -> Self {
        Self {
            author: Author::Bot,
            text: text.into(),
        }
    }

    pub fn user(text: impl Into<String>) -> Self {
        Self {
            author: Author::User,
            text: text.into(),
        }
    }
}

fn logfile() -> anyhow::Result<PathBuf> {
    let dir = ProjectDirs::from("", "bddap", PROJECT_NAME).ok_or(anyhow::anyhow!(
        "Could not find the config directory for the application."
    ))?;
    let dir = dir.data_dir();
    create_dir_all(dir).context("Could not create the config directory.")?;
    Ok(dir.join("convo.jsonl"))
}

pub fn store_message(message: LogMessage) -> anyhow::Result<()> {
    let message_path = logfile()?;
    let mut message_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(message_path)
        .context("Could not open the message log file.")?;

    match message.author {
        Author::User => eprintln!("{}: {}", "user".green(), message.text),
        Author::Bot => eprintln!("{}: {}", "ushidashi".blue(), message.text),
    }

    message_file.write_all(serde_json::to_string(&message)?.as_bytes())?;
    message_file.write_all(b"\n")?;

    Ok(())
}

pub fn load_messages() -> anyhow::Result<Vec<LogMessage>> {
    let message_path = logfile()?;
    let message_file = File::open(message_path);
    let reader = match message_file {
        Ok(file) => BufReader::new(file),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.into()),
    };

    let mut ret = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let message: LogMessage = serde_json::from_str(&line)
            .with_context(|| format!("Error parsing message on line {i}"))?;
        ret.push(message);
    }
    Ok(ret)
}
