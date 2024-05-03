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
use thiserror::Error;
use time::TimeError;
use unwrap_infallible::UnwrapInfallible;

use crate::time::Repeat;

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProcrastinationFileData(HashMap<String, Procrastination>);

impl ProcrastinationFileData {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn notify_all(&mut self) -> Result<(), NotificationError> {
        for procrastination in self.0.values_mut() {
            procrastination.notify()?;
        }
        Ok(())
    }

    /// delete already send notifications that are Timing::Once
    pub fn cleanup(&mut self) {
        self.0.retain(|_k, v| v.dirty != Dirt::Delete);
    }

    pub fn get(&self, k: &str) -> Option<&Procrastination> {
        self.0.get(k)
    }

    pub fn get_mut(&mut self, k: &str) -> Option<&mut Procrastination> {
        self.0.get_mut(k)
    }

    pub fn insert(&mut self, k: String, v: Procrastination) -> Option<Procrastination> {
        self.0.insert(k, v)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Procrastination {
    pub title: String,
    pub message: String,
    pub timing: Repeat,
    pub timestamp: DateTime<Local>,
    #[serde(skip)]
    dirty: Dirt,
}

impl Procrastination {
    pub fn new(title: String, message: String, timing: Repeat) -> Self {
        Procrastination {
            title,
            message,
            timing,
            timestamp: Local::now(),
            dirty: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Dirt {
    Clean,
    Update,
    Delete,
}

impl Default for Dirt {
    fn default() -> Self {
        Dirt::Clean
    }
}

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Could not deliver notification")]
    Notification(#[from] notify_rust::error::Error),
    #[error("invalid timing information for notification")]
    InvalidTiming(#[from] TimeError),
}

impl Procrastination {
    pub fn notify(&mut self) -> Result<(), NotificationError> {
        if !self.should_notify()? {
            return Ok(());
        }
        Notification::new()
            .summary(&self.title)
            .body(&self.message)
            .show()?;

        self.dirty = match &self.timing {
            Repeat::Once { timing: _ } => Dirt::Delete,
            Repeat::Repeat { timing: _ } => {
                self.timestamp = Local::now();
                Dirt::Update
            }
        };
        Ok(())
    }

    pub fn should_notify(&self) -> Result<bool, TimeError> {
        let last_timestamp = self.timestamp.naive_local();
        let next_notification = match &self.timing {
            Repeat::Once { timing } => match &timing {
                time::OnceTiming::Instant(instant) => instant.notification_date()?,
                time::OnceTiming::Delay(delay) => last_timestamp + *delay,
            },
            Repeat::Repeat { timing } => match &timing {
                time::RepeatTiming::Exact(e) => e.notification_date()?,
                time::RepeatTiming::Delay(d) => last_timestamp + *d,
            },
        };
        Ok(next_notification > last_timestamp && Local::now().naive_local() > next_notification)
    }
}

pub struct ProcrastinationFile {
    data: ProcrastinationFileData,
    lock: FileLock,
}

pub const FILE_NAME: &'static str = "procrastination.ron";
pub const DEFAULT_LOCATION: &'static str = ".local/share";

pub fn data_dir_path() -> PathBuf {
    if let Ok(config) = env::var("XDG_DATA_HOME") {
        PathBuf::from_str(&config).unwrap_infallible()
    } else {
        let home = env::var("HOME").expect("neither XDG_DATA_HOME nor HOME are set");
        let home = PathBuf::from_str(&home).unwrap_infallible();
        home.join(DEFAULT_LOCATION)
    }
}

pub fn procrastination_path(is_local: bool, path: Option<&PathBuf>) -> std::io::Result<PathBuf> {
    let path: PathBuf = if is_local {
        let current_dir = env::current_dir()?;
        current_dir.join(FILE_NAME)
    } else if let Some(file) = path {
        file.clone()
    } else {
        let config_dir = data_dir_path();
        config_dir.join(FILE_NAME)
    };
    Ok(path)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error on file-open {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to parse file {0}")]
    Parse(#[from] ron::error::SpannedError),
    #[error("Failed to serialize data")]
    Serialization(#[from] ron::Error),
}

impl ProcrastinationFile {
    pub fn new(data: ProcrastinationFileData, lock: FileLock) -> Self {
        Self { data, lock }
    }

    pub fn open(path: &PathBuf) -> Result<Self, Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let options = FileOptions::new().read(true).append(true);
        let mut lock = FileLock::lock(path, true, options)?;

        let mut content = String::new();
        lock.file.read_to_string(&mut content)?;

        let data = ron::from_str(&content)?;

        Ok(Self { data, lock })
    }

    pub fn data(&self) -> &ProcrastinationFileData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ProcrastinationFileData {
        &mut self.data
    }

    pub fn save(&mut self) -> Result<(), Error> {
        self.lock.file.set_len(0)?;

        ron::ser::to_writer_pretty(&mut self.lock.file, &self.data, PrettyConfig::default())?;

        self.lock.file.flush()?;
        Ok(())
    }
}
