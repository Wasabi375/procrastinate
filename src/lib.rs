pub mod arg_help;
pub mod nom_ext;
pub mod time;

use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::{DateTime, Local, NaiveDateTime};
use file_lock::{FileLock, FileOptions};
use notify_rust::Notification;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{OnceTiming, TimeError};
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
    pub fn cleanup(&mut self) -> bool {
        let mut changed = false;
        self.0.retain(|_k, v| {
            let retain = v.dirty != Dirt::Delete;
            if !retain {
                changed = true;
            }
            retain
        });
        changed
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

    pub fn remove(&mut self, key: &str) -> Option<Procrastination> {
        self.0.remove(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Procrastination)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Procrastination)> {
        self.0.iter_mut()
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
    #[serde(default)]
    pub sticky: bool,
    #[serde(default)]
    pub sleep: Option<Sleep>,
}

impl Procrastination {
    pub fn new(title: String, message: String, timing: Repeat, sticky: bool) -> Self {
        Procrastination {
            title,
            message,
            timing,
            timestamp: Local::now(),
            dirty: Default::default(),
            sticky,
            sleep: None,
        }
    }

    pub fn can_notify_in_future(&self) -> bool {
        self.dirty != Dirt::Delete
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sleep {
    pub timing: OnceTiming,
}

#[derive(Debug, PartialEq, Eq, Default)]
enum Dirt {
    #[default]
    Clean,
    Update,
    Delete,
}

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Could not deliver notification")]
    Notification(#[from] notify_rust::error::Error),
    #[error("invalid timing information for notification")]
    InvalidTiming(#[from] TimeError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum NotificationType {
    Normal,
    Sleep,
    None,
}

impl NotificationType {
    pub fn changed(&self) -> bool {
        match self {
            Self::Normal | Self::Sleep => true,
            Self::None => false,
        }
    }
}

impl Procrastination {
    pub fn notify(&mut self) -> Result<NotificationType, NotificationError> {
        let not_type = self.should_notify()?;
        if not_type == NotificationType::None {
            return Ok(not_type);
        }

        log::info!("Notification:\n{}\n\n{}", self.title, self.message);
        let mut notification = Notification::new();
        notification.summary(&self.title).body(&self.message);

        if self.sticky {
            notification.hint(notify_rust::Hint::Resident(true));
            notification.timeout(0);
        }

        notification.show()?;

        self.sleep = None;

        self.dirty = match &self.timing {
            Repeat::Once { timing: _ } => Dirt::Delete,
            Repeat::Repeat { timing: _ } => {
                self.timestamp = Local::now();
                Dirt::Update
            }
        };
        Ok(not_type)
    }

    pub fn should_notify(&self) -> Result<NotificationType, TimeError> {
        let last_timestamp = self.timestamp.naive_local();
        let (typ, next_notification) = self.next_notification()?;
        if next_notification > last_timestamp && Local::now().naive_local() > next_notification {
            Ok(typ)
        } else {
            Ok(NotificationType::None)
        }
    }

    pub fn next_notification(&self) -> Result<(NotificationType, NaiveDateTime), TimeError> {
        let last_timestamp = self.timestamp.naive_local();
        let next_notification = match &self.timing {
            Repeat::Once { timing } => next_once_timing(timing, last_timestamp)?,
            Repeat::Repeat { timing } => next_repeat_timing(timing, last_timestamp)?,
        };

        if let Some(sleep) = self.sleep.as_ref() {
            let next_sleep_notification = next_once_timing(&sleep.timing, last_timestamp)?;
            if next_sleep_notification < next_notification {
                Ok((NotificationType::Sleep, next_sleep_notification))
            } else {
                Ok((NotificationType::Normal, next_notification))
            }
        } else {
            Ok((NotificationType::Normal, next_notification))
        }
    }
}

fn next_repeat_timing(
    timing: &time::RepeatTiming,
    last_timestamp: NaiveDateTime,
) -> Result<NaiveDateTime, TimeError> {
    Ok(match timing {
        time::RepeatTiming::Exact(e) => e.notification_date()?,
        time::RepeatTiming::Delay(delay) => last_timestamp + *delay,
    })
}

fn next_once_timing(
    timing: &OnceTiming,
    last_timestamp: NaiveDateTime,
) -> Result<NaiveDateTime, TimeError> {
    Ok(match timing {
        time::OnceTiming::Instant(instant) => instant.notification_date()?,
        time::OnceTiming::Delay(delay) => last_timestamp + *delay,
    })
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

    pub fn open(path: &Path) -> Result<Self, Error> {
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

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::{
        time::{Repeat, RepeatTiming},
        Procrastination,
    };

    #[test]
    fn can_deser_0_3_2_procrastination() {
        let input = r#"(
            title: "NixOs update required",
            message: "It has been a month since the last update",
            timing: Repeat(
                timing: Delay((
                    secs: 2592000,
                    nanos: 0,
                )),
            ),
            timestamp: "2024-09-12T04:41:38.864837768+02:00",
        )"#;
        let proc: Procrastination =
            ron::from_str(input).expect("Failed to parse proc data from version 0.3.2");

        assert_eq!(proc.title, "NixOs update required");
        assert_eq!(proc.message, "It has been a month since the last update");
        assert_eq!(
            proc.timing,
            Repeat::Repeat {
                timing: RepeatTiming::Delay(Duration::from_secs(2592000))
            }
        )
    }
}
