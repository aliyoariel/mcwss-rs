//! Console + file logger. Only `INFO` and above is written to the log file.

use std::{fs::OpenOptions, io::Write, path::PathBuf};

use chrono::Local;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace = 10,
    Debug = 20,
    Info = 30,
    Warn = 40,
    Error = 50,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }

    fn color(self) -> &'static str {
        match self {
            Level::Trace => "\x1b[90m",
            Level::Debug => "\x1b[36m",
            Level::Info => "\x1b[32m",
            Level::Warn => "\x1b[33m",
            Level::Error => "\x1b[31m",
        }
    }
}

pub struct Logger {
    level: Level,
    dir: PathBuf,
}

impl Logger {
    pub fn new(level: Level, dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&dir);
        Self { level, dir }
    }

    pub fn log(&self, level: Level, message: &str) {
        if level < self.level {
            return;
        }
        let now = Local::now();
        println!(
            "{} {}{:<5}\x1b[0m {}",
            now.format("%H:%M:%S"),
            level.color(),
            level.label(),
            message
        );
        if level >= Level::Info {
            let path = self.dir.join(format!("{}.log", now.format("%Y-%m-%d")));
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(
                    file,
                    "{} {:<5} {}",
                    now.format("%Y-%m-%d %H:%M:%S"),
                    level.label(),
                    message
                );
            }
        }
    }

    pub fn debug(&self, message: &str) {
        self.log(Level::Debug, message);
    }

    pub fn info(&self, message: &str) {
        self.log(Level::Info, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(Level::Warn, message);
    }

    pub fn error(&self, message: &str) {
        self.log(Level::Error, message);
    }
}
