use std::error::Error;

use args::Args;
use clap::Parser;
use procrastinate::{procrastination_path, ProcrastinationFile};

mod args;

fn main() -> Result<(), Box<dyn Error>> {
    #[allow(unused_mut)]
    let mut args = Args::parse();

    #[cfg(debug_assertions)]
    {
        if std::env::var("PROCRASTINATE_DEBUG_LOCAL").is_ok() {
            args.local = true;
            eprintln!("local debug override active");
        }
    }

    if args.verbose {
        println!("args: {args:?}");
    }

    let path = procrastination_path(args.local, args.file.as_ref())?;
    let mut procrastination =
        ProcrastinationFile::open(&path).expect("could not open procrastination file");

    if let Some(key) = args.key.as_ref() {
        if let Some(procrastination) = procrastination.data_mut().get_mut(key) {
            procrastination.notify()?;
        } else {
            panic!("No procrastination with key \"{key}\" found");
        }
    } else {
        procrastination.data_mut().notify_all()?;
    }
    procrastination.data_mut().cleanup();
    procrastination.save()?;

    Ok(())
}
