use chrono::Local;
use file_lock::{FileLock, FileOptions};
use procrastinate::{
    procrastination_path, Procrastination, ProcrastinationFile, ProcrastinationFileData,
};

use crate::args::Args;

pub mod args;

fn open_or_create(args: &Args) -> ProcrastinationFile {
    let local = args.local;
    let path_buf = args.file.as_ref();
    let path = procrastination_path(local, path_buf);

    let (data, lock) = if path.exists() {
        ProcrastinationFile::open(&path)
    } else {
        let data = ProcrastinationFileData::empty();
        let options = FileOptions::new().create_new(true).write(true);
        let lock = FileLock::lock(&path, true, options).expect("Failed to take file lock");
        (data, lock)
    };
    ProcrastinationFile::new(data, lock)
}

fn procrastination(args: &Args) -> Procrastination {
    Procrastination {
        title: args.title.clone().unwrap_or(args.key.clone()),
        message: args.message.clone().unwrap_or(String::new()),
        timing: args.timing.clone(),
        timestamp: Local::now(),
    }
}

fn main() {
    use clap::Parser;
    let args = Args::parse();
    args.verify();

    if args.verbose {
        println!("args: {args:?}");
    }

    let mut procrastination_file = open_or_create(&args);
    procrastination_file
        .data_mut()
        .0
        .insert(args.key.clone(), procrastination(&args));
    procrastination_file.save();
}
