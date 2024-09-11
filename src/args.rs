use core::panic;
use std::path::PathBuf;

use clap::{Args, Parser};
use procrastinate::{
    arg_help::{ONCE_TIMING_ARG_DOC, REPEAT_TIMING_ARG_DOC},
    file_arg_doc, local_arg_doc,
    time::{OnceTiming, Repeat, RepeatTiming},
    Procrastination,
};

#[derive(Parser, Debug)]
#[command(version, about)]
/// Create a new procrastination.
///
/// Either `procrastinate-daemon` or `procrastinate-work` can notify you
/// when it's time to stop procrastinating on the given taks.
pub struct Arguments {
    /// A key to identify this procrastination
    pub key: String,

    /// How long to procrastinate for
    #[command(subcommand)]
    pub cmd: Cmd,

    #[arg(short, long, help = local_arg_doc!())]
    pub local: bool,

    /// procrastinate at file
    #[arg(short, long, help = file_arg_doc!())]
    pub file: Option<PathBuf>,

    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Args, Clone)]
pub struct NotificationArgs {
    /// the title that will be displayed when the procrastination is over.
    ///
    /// Defaults to `key`
    #[arg(short, long)]
    pub title: Option<String>,

    /// A short message that will be displayed when the procrastination is over
    #[arg(short, long)]
    pub message: Option<String>,
}

impl Arguments {
    pub fn verify(&self) -> Result<(), String> {
        if self.local && self.file.is_some() {
            return Err(format!("'local' and 'file' are mutually exclusive"));
        }
        Ok(())
    }

    pub fn procrastination(&self) -> Procrastination {
        let (args, timing) = match &self.cmd {
            Cmd::Once { timing, args } => (
                args,
                Repeat::Once {
                    timing: timing.clone(),
                },
            ),
            Cmd::Repeat { timing, args } => (
                args,
                Repeat::Repeat {
                    timing: timing.clone(),
                },
            ),
            Cmd::Done => panic!("can't create new procrastination from done cmd"),
        };
        Procrastination::new(
            args.title.clone().unwrap_or(self.key.clone()),
            args.message.clone().unwrap_or(String::new()),
            timing,
        )
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Cmd {
    /// Procrastinating on any taks is great
    Once {
        #[arg(help = ONCE_TIMING_ARG_DOC)]
        timing: OnceTiming,
        #[command(flatten)]
        args: NotificationArgs,
    },
    /// procrastination is only great when doing it again and again
    Repeat {
        #[arg(help = REPEAT_TIMING_ARG_DOC)]
        timing: RepeatTiming,
        #[command(flatten)]
        args: NotificationArgs,
    },
    /// stop procrastinating on a given taks
    Done,
}
