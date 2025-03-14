use std::path::PathBuf;

use clap::Parser;

use procrastinate::{check_key_arg_doc, file_arg_doc, local_arg_doc};

#[derive(Parser, Debug)]
#[command(version, about)]
/// Continously checks notifications for all finished procrastinations.
///
/// To only check for notifications once use `procrastinate-work` insetead.
pub struct Args {
    #[arg(help = check_key_arg_doc!())]
    pub key: Option<String>,

    #[arg(short, long, help = local_arg_doc!())]
    pub local: bool,

    /// minimum time to wait before checking pending notifications in seconds
    #[arg(short, long, default_value_t = 1)]
    pub min: u64,

    /// max time to wait before checking pending notifications in seconds
    #[arg(short('M'), long, default_value_t = 300)]
    pub max: u64,

    /// procrastinate at file
    #[arg(short, long, help = file_arg_doc!())]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub verbose: bool,
}
