use chrono::{NaiveDateTime, NaiveTime};
use nom::combinator::complete;
use std::{str::FromStr, time::Duration};

mod parsing;

use parsing::{parse_duration, parse_repeat_exact, parse_rough_instant};

use crate::nom_ext::consume_all;

const SECONDS_IN_HOUR: u64 = 60 * 60;
const SECONDS_IN_DAY: u64 = SECONDS_IN_HOUR * 24;
const SECONDS_IN_WEEK: u64 = SECONDS_IN_DAY * 7;
const SECONDS_IN_MONTH: u64 = SECONDS_IN_DAY * 30;
const SECONDS_IN_YEAR: u64 = SECONDS_IN_DAY * 365;

pub const DAYS_IN_WEEK: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

pub const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoughInstant {
    DayOfMonth { day: u8, time: Option<NaiveTime> },
    DayOfWeek { day: u8, time: Option<NaiveTime> },
    Today { time: NaiveTime },
    Tomorrow { time: Option<NaiveTime> },
    Date { date: NaiveDateTime },
    Month { month: u8 },
}

impl FromStr for RoughInstant {
    type Err = nom::Err<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match consume_all(parse_rough_instant)(s) {
            Ok((_, instant)) => Ok(instant),
            Err(error) => match error {
                nom::Err::Incomplete(err) => Err(nom::Err::Incomplete(err)),
                nom::Err::Error(err) => Err(nom::Err::Error(err.to_string())),
                nom::Err::Failure(err) => Err(nom::Err::Failure(err.to_string())),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepeatExact {
    DayOfMonth {
        /// 0 index into year starting with january
        day: u8,

        time: Option<NaiveTime>,
    },
    DayOfWeek {
        /// 0 index into week starting with monda
        day: u8,
        time: Option<NaiveTime>,
    },
    Daily {
        time: Option<NaiveTime>,
    },
}

impl FromStr for RepeatExact {
    type Err = nom::Err<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match consume_all(parse_repeat_exact)(s) {
            Ok((_, repeat)) => Ok(repeat),
            Err(error) => match error {
                nom::Err::Incomplete(err) => Err(nom::Err::Incomplete(err)),
                nom::Err::Error(err) => Err(nom::Err::Error(err.to_string())),
                nom::Err::Failure(err) => Err(nom::Err::Failure(err.to_string())),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delay(Duration);

impl FromStr for Delay {
    type Err = nom::Err<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match consume_all(parse_duration)(s) {
            Ok((_, duration)) => Ok(Delay(duration)),
            Err(error) => match error {
                nom::Err::Incomplete(err) => Err(nom::Err::Incomplete(err)),
                nom::Err::Error(err) => Err(nom::Err::Error(err.to_string())),
                nom::Err::Failure(err) => Err(nom::Err::Failure(err.to_string())),
            },
        }
    }
}
