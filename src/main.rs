// TODO temp
#![allow(dead_code)]

use core::panic;
use std::{path::PathBuf, str::FromStr, time::Duration};

use nom::{branch::alt, IResult};
use nom_ext::consume_all;
use time::{RepeatExact, RoughInstant};

use crate::time::parsing::{parse_duration, parse_rough_instant};

mod nom_ext;
mod time;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A key to identify this procrastination
    key: String,

    /// A short message that will be displayed when the procrastination is over
    #[arg(short, long)]
    message: Option<String>,

    /// How long to procrastinate for
    #[command(subcommand)]
    timing: Repeat,

    /// procrastinate in current working directory
    #[arg(short, long)]
    local: bool,

    /// procrastinate at file
    #[arg(short, long)]
    file: Option<PathBuf>,
}

impl Args {
    fn verify(&self) {
        if self.local && self.file.is_some() {
            panic!("'local' and 'file' are mutually exclusive");
        }
    }
}

#[derive(clap::Subcommand, Debug)]
enum Repeat {
    /// only procrastinate once
    Once { timing: OnceTiming },
    /// procrastination is only great when doing it again and again
    Repeat { timing: RepeatTiming },
}

#[derive(Debug, Clone)]
enum OnceTiming {
    Instant(RoughInstant),
    Delay(Duration),
}

fn parse_once_instant(input: &str) -> IResult<&str, OnceTiming> {
    let (input, instant) = parse_rough_instant(input)?;
    Ok((input, OnceTiming::Instant(instant)))
}

fn parse_once_delay(input: &str) -> IResult<&str, OnceTiming> {
    let (input, delay) = parse_duration(input)?;
    Ok((input, OnceTiming::Delay(delay)))
}

impl FromStr for OnceTiming {
    type Err = nom::Err<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match consume_all(alt((parse_once_instant, parse_once_delay)))(s) {
            Ok((_, once)) => Ok(once),
            Err(error) => match error {
                nom::Err::Incomplete(err) => Err(nom::Err::Incomplete(err)),
                nom::Err::Error(err) => Err(nom::Err::Error(err.to_string())),
                nom::Err::Failure(err) => Err(nom::Err::Failure(err.to_string())),
            },
        }
    }
}

#[derive(Debug, Clone)]
enum RepeatTiming {
    Exact(RepeatExact),
    Delay(Duration),
}

fn parse_repeat_exact(input: &str) -> IResult<&str, RepeatTiming> {
    let (input, exact) = time::parsing::parse_repeat_exact(input)?;
    Ok((input, RepeatTiming::Exact(exact)))
}

fn parse_repeat_delay(input: &str) -> IResult<&str, RepeatTiming> {
    let (input, delay) = parse_duration(input)?;
    Ok((input, RepeatTiming::Delay(delay)))
}

impl FromStr for RepeatTiming {
    type Err = nom::Err<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match consume_all(alt((parse_repeat_exact, parse_repeat_delay)))(s) {
            Ok((_, repeat)) => Ok(repeat),
            Err(error) => match error {
                nom::Err::Incomplete(err) => Err(nom::Err::Incomplete(err)),
                nom::Err::Error(err) => Err(nom::Err::Error(err.to_string())),
                nom::Err::Failure(err) => Err(nom::Err::Failure(err.to_string())),
            },
        }
    }
}

fn main() {
    use clap::Parser;
    let args = Args::parse();

    println!("args: {args:?}");
}
