use core::panic;
use std::{ops::Deref, path::PathBuf};

use clap::{Args, Parser};
use procrastinate::{
    arg_help::{ONCE_TIMING_ARG_DOC, REPEAT_TIMING_ARG_DOC},
    file_arg_doc, local_arg_doc,
    time::{OnceTimingPart, Repeat, RepeatTimingPart},
    Procrastination,
};

#[derive(Parser, Debug)]
#[command(version, about)]
/// Create a new procrastination.
///
/// Either `procrastinate-daemon` or `procrastinate-work` can notify you
/// when it's time to stop procrastinating on the given taks.
pub struct Arguments {
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
            return Err("'local' and 'file' are mutually exclusive".to_string());
        }
        Ok(())
    }

    pub fn procrastination(&self) -> Procrastination {
        let (key, args, timing, sticky) = match &self.cmd {
            Cmd::Once {
                key,
                timing,
                args,
                sticky,
            } => (
                key,
                args,
                Repeat::Once {
                    timing: timing.deref().try_into().unwrap(),
                },
                sticky,
            ),
            Cmd::Repeat {
                key,
                timing,
                args,
                sticky,
            } => (
                key,
                args,
                Repeat::Repeat {
                    timing: timing.deref().try_into().unwrap(),
                },
                sticky,
            ),
            Cmd::Done { .. } | Cmd::List { .. } | Cmd::Sleep { .. } => {
                panic!("can't create new procrastination from done, list or sleep cmd")
            }
        };
        Procrastination::new(
            args.title.clone().unwrap_or(key.clone()),
            args.message.clone().unwrap_or(String::new()),
            timing,
            *sticky,
        )
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Cmd {
    /// Procrastinating on any taks is great
    Once {
        /// A key to identify this procrastination
        key: String,

        #[arg(help = ONCE_TIMING_ARG_DOC, trailing_var_arg = true)]
        timing: Vec<OnceTimingPart>,

        #[command(flatten)]
        args: NotificationArgs,
        /// If set any any notification must be explicitly dismissed
        #[arg(short, long)]
        sticky: bool,
    },
    /// procrastination is only great when doing it again and again
    Repeat {
        /// A key to identify this procrastination
        key: String,

        #[arg(help = REPEAT_TIMING_ARG_DOC, trailing_var_arg = true)]
        timing: Vec<RepeatTimingPart>,

        #[command(flatten)]
        args: NotificationArgs,

        /// If set any any notification must be explicitly dismissed
        #[arg(short, long)]
        sticky: bool,
    },
    /// stop procrastinating on a given taks
    Done {
        /// A key to identify this procrastination
        key: String,
    },
    /// List all tasks you are procrastinating
    List {
        /// print the procrastination list using rust debug print
        #[arg(long, short)]
        debug: bool,

        /// print the procrastination list in the ron format
        #[arg(long, short)]
        ron: bool,

        /// print dates with the wrong month.day format
        /// instead of the sensible day.month format
        #[arg(long, short)]
        us_date: bool,
    },
    Sleep {
        /// A key to identify this procrastination
        key: String,
        #[arg(help = ONCE_TIMING_ARG_DOC, trailing_var_arg = true)]
        timing: Vec<OnceTimingPart>,
    },
}
