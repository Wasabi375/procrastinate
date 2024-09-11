use std::{error::Error, path::PathBuf};

use clap::Parser;
use procrastinate::{
    check_key_arg_doc, file_arg_doc, local_arg_doc, procrastination_path, ProcrastinationFile,
};

#[derive(Parser, Debug)]
#[command(version, about)]
/// Shows notifications for all finished procrastinations.
///
/// This will not wait for any procrastinations to be finished.
/// If you want to continously notify when procrastinations finish
/// you can use `procrastinate-daemon` instead.
pub struct Args {
    #[arg(help =  check_key_arg_doc!())]
    pub key: Option<String>,

    #[arg(short, long, help = local_arg_doc!())]
    pub local: bool,

    /// Check for procrastinations in the given file.
    ///
    /// This is ignored if `local` is set.
    #[arg(short, long, help = file_arg_doc!())]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub verbose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    #[allow(unused_mut)]
    let mut args = Args::parse();

    #[cfg(debug_assertions)]
    {
        if std::env::var("PROCRASTINATE_DEBUG_LOCAL").is_ok() {
            args.local = true;
            if args.verbose {
                println!("local debug override active");
            }
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
