// TODO temp
#![allow(dead_code)]

use time::{Delay, RepeatExact, RoughInstant};

mod nom_ext;
mod time;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    key: String,

    #[command(subcommand)]
    timing: Timing,
}

#[derive(clap::Subcommand, Debug)]
enum Timing {
    Instant { instant: RoughInstant },
    Delay { delay: Delay },
    RepeatExact { timing: RepeatExact },
    RepeatDelay { delay: Delay },
}

fn main() {
    use clap::Parser;
    let args = Args::parse();

    println!("args: {args:?}");
}
