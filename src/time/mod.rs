use std::{str::FromStr, time::Duration};

use chrono::{Datelike, Days, Local, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use nom::{branch::alt, IResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::nom_ext::consume_all;

use self::parsing::{parse_duration, parse_rough_instant};

pub mod parsing;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Repeat {
    Once { timing: OnceTiming },
    Repeat { timing: RepeatTiming },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OnceTiming {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepeatTiming {
    Exact(RepeatExact),
    Delay(Duration),
}

fn parse_repeat_exact(input: &str) -> IResult<&str, RepeatTiming> {
    let (input, exact) = parsing::parse_repeat_exact(input)?;
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoughInstant {
    DayOfMonth {
        day: u8,
        time: Option<NaiveTime>,
    },
    DayOfWeek {
        /// Mon = 0, Tue = 1, etc
        day: u8,
        time: Option<NaiveTime>,
    },
    Date {
        date: NaiveDateTime,
    },
    Month {
        month: u8,
    },
}

#[derive(Debug, Error)]
pub enum TimeError {
    #[error("{0} is not a valid day")]
    InvalidDay(u8),
    #[error("{0} is not a valid month")]
    InvalidMonth(u8),
}

fn monday_same_week(date: &NaiveDate) -> NaiveDate {
    let days_since_mon = date.weekday().days_since(Weekday::Mon);
    *date - Days::new(days_since_mon.into())
}

impl RoughInstant {
    pub fn notification_date(&self) -> Result<NaiveDateTime, TimeError> {
        let now = Local::now().naive_local();
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        match self {
            RoughInstant::DayOfMonth { day, time } => Ok(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(now.year(), now.month(), *day as u32)
                    .ok_or(TimeError::InvalidDay(*day))?,
                time.unwrap_or(midnight),
            )),
            RoughInstant::DayOfWeek { day, time } => {
                let today = now.date();
                let week_start = monday_same_week(&today);
                let day = week_start + Days::new((*day).into());
                Ok(NaiveDateTime::new(day, time.clone().unwrap_or(midnight)))
            }
            RoughInstant::Date { date } => Ok(date.clone()),
            RoughInstant::Month { month } => Ok(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(now.year(), *month as u32, 1)
                    .ok_or(TimeError::InvalidMonth(*month))?,
                midnight,
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

impl RepeatExact {
    pub fn notification_date(&self) -> Result<NaiveDateTime, TimeError> {
        let now = Local::now().naive_local();
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        match self {
            RepeatExact::DayOfMonth { day, time } => Ok(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(now.year(), now.month(), *day as u32)
                    .ok_or(TimeError::InvalidDay(*day))?,
                time.unwrap_or(midnight),
            )),
            RepeatExact::DayOfWeek { day, time } => {
                let today = now.date();
                let week_start = monday_same_week(&today);
                let day = week_start + Days::new((*day).into());
                Ok(NaiveDateTime::new(day, time.clone().unwrap_or(midnight)))
            }

            RepeatExact::Daily { time } => {
                let today = now.date();
                Ok(NaiveDateTime::new(today, time.unwrap_or(midnight)))
            }
        }
    }
}
