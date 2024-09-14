use file_lock::{FileLock, FileOptions};
use procrastinate::{
    procrastination_path, Error, ProcrastinationFile, ProcrastinationFileData, Sleep,
};

use crate::args::{Arguments, Cmd};

pub mod args;

fn open_or_create(args: &Arguments) -> Result<ProcrastinationFile, Error> {
    let local = args.local;
    let path_buf = args.file.as_ref();
    let path = procrastination_path(local, path_buf)?;

    if path.exists() {
        ProcrastinationFile::open(&path)
    } else {
        let data = ProcrastinationFileData::empty();
        let options = FileOptions::new().create_new(true).write(true);
        let lock = FileLock::lock(&path, true, options)?;
        Ok(ProcrastinationFile::new(data, lock))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    #[allow(unused_mut)]
    let mut args = Arguments::parse();
    args.verify()?;

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

    let mut procrastination_file = open_or_create(&args)?;

    match args.cmd {
        Cmd::Once { ref key, .. } | Cmd::Repeat { ref key, .. } => {
            procrastination_file
                .data_mut()
                .insert(key.clone(), args.procrastination());
        }
        Cmd::Done { ref key } => {
            procrastination_file.data_mut().remove(key);
        }
        Cmd::List => {
            for proc in procrastination_file.data().iter() {
                // TODO print this for user instead of debug
                println!("{}: {:#?}", proc.0, proc.1);
            }
        }
        Cmd::Sleep { ref key, timing } => {
            if let Some(proc) = procrastination_file.data_mut().get_mut(key) {
                proc.sleep = Some(Sleep { timing });
            } else {
                println!("No procrastination entry with key \"{key}\" exists");
            }
        }
    };

    procrastination_file.save()?;

    Ok(())
}
