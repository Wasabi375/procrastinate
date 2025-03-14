use std::path::PathBuf;

use clap::Parser;
use procrastinate::{check_key_arg_doc, file_arg_doc, local_arg_doc};

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
