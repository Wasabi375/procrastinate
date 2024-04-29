use std::path::PathBuf;

use clap::Parser;

use procrastinate::time::Repeat;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// A key to identify this procrastination
    pub key: String,

    /// The title that will be displayed when the procrastination is over.
    ///
    /// Defaults to `key`
    #[arg(short, long)]
    pub title: Option<String>,

    /// A short message that will be displayed when the procrastination is over
    #[arg(short, long)]
    pub message: Option<String>,

    /// How long to procrastinate for
    #[command(subcommand)]
    pub timing: Repeat,

    /// procrastinate in current working directory
    #[arg(short, long)]
    pub local: bool,

    /// procrastinate at file
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    pub fn verify(&self) {
        if self.local && self.file.is_some() {
            panic!("'local' and 'file' are mutually exclusive");
        }
    }
}
