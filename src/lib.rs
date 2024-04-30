pub mod nom_ext;
pub mod time;

use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use chrono::{DateTime, Local};
use file_lock::{FileLock, FileOptions};
use notify_rust::Notification;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::time::Repeat;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcrastinationFileData(pub HashMap<String, Procrastination>);

impl ProcrastinationFileData {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn notify_all(&self) {
        for procrastination in self.0.values() {
            procrastination.notify();
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Procrastination {
    pub title: String,
    pub message: String,
    pub timing: Repeat,
    pub timestamp: DateTime<Local>,
}

impl Procrastination {
    pub fn notify(&self) {
        if !self.should_notify() {
            return;
        }
        Notification::new()
            .summary(&self.title)
            .body(&self.message)
            .show()
            .expect("failed to send message");
    }

    pub fn should_notify(&self) -> bool {
        let last_timestamp = self.timestamp.naive_local();
        let next_notification = match &self.timing {
            Repeat::Once { timing } => match &timing {
                time::OnceTiming::Instant(i) => i.notification_date(),
                time::OnceTiming::Delay(d) => last_timestamp + *d,
            },
            Repeat::Repeat { timing } => match &timing {
                time::RepeatTiming::Exact(e) => e.notification_date(),
                time::RepeatTiming::Delay(d) => last_timestamp + *d,
            },
        };
        next_notification > last_timestamp && Local::now().naive_local() > next_notification
    }
}

pub struct ProcrastinationFile {
    data: ProcrastinationFileData,
    lock: FileLock,
}

pub const FILE_NAME: &'static str = "procrastination.ron";
pub const DEFAULT_LOCATION: &'static str = ".config/";

pub fn config_dir_path() -> PathBuf {
    if let Ok(config) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from_str(&config).expect("XDG_CONFIG_HOME value can't be parsed as PathBuf")
    } else {
        let home = env::var("HOME").expect("neither XDG_CONFIG_HOME nor HOME are set");
        let home = PathBuf::from_str(&home).expect("HOME value can't be parsed as PathBuf");
        home.join(DEFAULT_LOCATION)
    }
}

pub fn procrastination_path(is_local: bool, path: Option<&PathBuf>) -> PathBuf {
    let path: PathBuf = if is_local {
        let current_dir = env::current_dir().expect("Could not get current working dir");
        current_dir.join(FILE_NAME)
    } else if let Some(file) = path {
        file.clone()
    } else {
        let config_dir = config_dir_path();
        config_dir.join(FILE_NAME)
    };
    path
}

impl ProcrastinationFile {
    pub fn new(data: ProcrastinationFileData, lock: FileLock) -> Self {
        Self { data, lock }
    }

    pub fn open(path: &PathBuf) -> Self {
        let options = FileOptions::new().read(true).append(true);
        let mut lock = FileLock::lock(path, true, options).expect("Failed to take file lock");

        let mut content = String::new();
        lock.file
            .read_to_string(&mut content)
            .expect("Failed to read file content");

        let data = ron::from_str(&content).expect("failed to parse procrastination file");

        Self { data, lock }
    }

    pub fn data(&self) -> &ProcrastinationFileData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ProcrastinationFileData {
        &mut self.data
    }

    pub fn save(&mut self) {
        self.lock
            .file
            .set_len(0)
            .expect("failed to clear file before refilling it");

        ron::ser::to_writer_pretty(&mut self.lock.file, &self.data, PrettyConfig::default())
            .expect("failed to write procrastination data");

        self.lock
            .file
            .flush()
            .expect("failed to write procrastination data");
    }
}
