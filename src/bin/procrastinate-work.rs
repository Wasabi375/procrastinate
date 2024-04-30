use std::path::PathBuf;

use clap::Parser;
use procrastinate::{procrastination_path, ProcrastinationFile};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub key: Option<String>,

    #[arg(short, long)]
    pub local: bool,

    /// procrastinate at file
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("args: {args:?}");
    }

    let path = procrastination_path(args.local, args.file.as_ref());
    let procrastination = ProcrastinationFile::open(&path);

    if let Some(key) = args.key.as_ref() {
        if let Some(procrastination) = procrastination.data().0.get(key) {
            procrastination.notify();
        } else {
            panic!("No procrastination with key \"{key}\" found");
        }
    } else {
        procrastination.data().notify_all();
    }
}
