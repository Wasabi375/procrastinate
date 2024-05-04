use file_lock::{FileLock, FileOptions};
use procrastinate::{procrastination_path, Error, ProcrastinationFile, ProcrastinationFileData};

use crate::args::Arguments;

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
            if args.verbose {
                println!("local debug override active");
            }
        }
    }

    if args.verbose {
        println!("args: {args:?}");
    }

    let mut procrastination_file = open_or_create(&args)?;
    procrastination_file
        .data_mut()
        .insert(args.key.clone(), args.procrastination());
    procrastination_file.save()?;

    Ok(())
}
