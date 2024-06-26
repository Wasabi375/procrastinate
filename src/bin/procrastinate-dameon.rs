use core::panic;
use std::{
    error::Error,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{Local, Timelike};
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use notify::{RecommendedWatcher, Watcher};
use notify_rust::Notification;
use procrastinate::{
    procrastination_path, save_on_write_file::SaveOnWriteFile, ProcrastinationFile,
};
use tokio::{pin, select, sync::watch};
use tokio_stream::{wrappers::WatchStream, StreamExt};

fn check_for_notifications(
    path: &Path,
    min: Duration,
    max: Duration,
) -> Result<Duration, Box<dyn std::error::Error>> {
    let mut proc_file = ProcrastinationFile::open(path)?;
    let now = Local::now().naive_local();
    log::info!("check for notifications");

    let mut until_any_next = Duration::MAX;
    let mut err = None;

    for (_key, procrastination) in proc_file.data_mut().iter_mut() {
        procrastination.notify()?;

        if !procrastination.can_notify_in_future() {
            continue;
        }

        match procrastination.next_notification() {
            Ok(next_notification_at) => {
                let until_next = next_notification_at - now;
                let until_next = until_next.to_std().unwrap_or(Duration::MAX);

                until_any_next = until_any_next.min(until_next);
            }
            Err(e) => {
                log::error!("Failed to find next notification: {e}");
                err = Some(e);
            }
        }
    }
    proc_file.data_mut().cleanup();

    proc_file.save()?;

    if let Some(err) = err {
        return Err(err.into());
    }

    log::info!("Next notification check in {:?}", until_any_next);
    Ok(until_any_next.clamp(min, max))
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub key: Option<String>,

    #[arg(short, long)]
    pub local: bool,

    /// minimum time to wait before checking pending notifications in seconds
    #[arg(short, long, default_value_t = 1)]
    pub min: u64,

    /// max time to wait before checking pending notifications in seconds
    #[arg(short('M'), long, default_value_t = 300)]
    pub max: u64,

    /// procrastinate at file
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub verbose: bool,
}

fn init_logger(verbose: bool) {
    let log_path: PathBuf = std::env::var("XDG_CACHE_HOME")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("Could not find home directory");
            format!("{home}/.cache")
        })
        .into();
    let now = Local::now().naive_local();
    let log_path = log_path.join("procrastinate").join(format!(
        "dameon-{}-{}-{}.log",
        now.date(),
        now.time().hour(),
        now.time().minute()
    ));
    if verbose {
        println!("log-path: {:?}", log_path);
    }
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).expect("Could not create log out dir");
    }
    let log_file = SaveOnWriteFile::new(log_path);

    let mut builder = Builder::new();
    if verbose {
        builder.filter_level(LevelFilter::Info);
    } else {
        builder.filter_level(LevelFilter::Error);
    }
    builder.parse_default_env();
    builder.target(env_logger::Target::Pipe(Box::new(log_file)));
    builder.init();
}

async fn work(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let min_dur = Duration::from_secs(args.min);
    let max_dur = Duration::from_secs(args.max);

    let path = procrastination_path(args.local, args.file.as_ref())?;

    let timeout = check_for_notifications(&path, min_dur, max_dur).unwrap_or(min_dur);
    let mut sleep = tokio::time::sleep(timeout);

    let (_file_watcher, mut file_watch) = watch(&path)?;
    let mut last_n_iters_failed = 0;

    loop {
        {
            // Wait for either timeout or file change
            pin!(sleep);
            select! {
                _ = &mut sleep => {}
                next = file_watch.next() => {
                    if next.is_none() {
                        let err: Box<dyn Error> = "File watch stream closed".into();
                        display_error_notification(err.as_ref());
                        return Err(err);
                    }
                }
            }
        }
        match check_for_notifications(&path, min_dur, max_dur) {
            Ok(timeout) => {
                sleep = tokio::time::sleep(timeout);
                last_n_iters_failed = 0;
            }
            Err(err) => {
                display_error_notification(err.as_ref());
                if last_n_iters_failed == 2 {
                    let err: Box<dyn Error> =
                        "Notification check failed to often. Stopping dameon".into();
                    display_error_notification(err.as_ref());
                    return Err(err);
                }
                last_n_iters_failed += 1;
                sleep = tokio::time::sleep(min_dur);
            }
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[allow(unused_mut)]
    let mut args = Args::parse();

    init_logger(args.verbose);

    #[cfg(debug_assertions)]
    {
        if std::env::var("PROCRASTINATE_DEBUG_LOCAL").is_ok() {
            args.local = true;
            if args.verbose {
                log::info!("local debug override active");
            }
        }
    }

    if args.verbose {
        log::info!("args: {args:?}");
    }

    match work(&args).await {
        Ok(o) => Ok(o),
        Err(e) => {
            log::error!("Dameon failed with: {e}");
            Err(e)
        }
    }
}

fn display_error_notification(err: &dyn Error) {
    Notification::new()
        .summary("Procrastinate-Dameon error")
        .body(&format!("{err}"))
        .show()
        .expect("failed to notify about previous error");
}

fn watch(path: &Path) -> notify::Result<(RecommendedWatcher, WatchStream<()>)> {
    let (tx, rx) = watch::channel(());

    let mut watcher = RecommendedWatcher::new(
        move |event: Result<notify::Event, notify::Error>| match event {
            Ok(event) => match event.kind {
                notify::EventKind::Any
                | notify::EventKind::Create(_)
                | notify::EventKind::Modify(_)
                | notify::EventKind::Other => tx.send(()).unwrap(),
                _ => {
                    // ignore non modify and delete events. Those don't change when we need to check next
                }
            },
            Err(err) => {
                log::error!("File watcher error: {err}");
                panic!("File watcher error: {err}")
            }
        },
        Default::default(),
    )?;
    watcher.watch(path, notify::RecursiveMode::Recursive)?;

    Ok((watcher, WatchStream::from_changes(rx)))
}
