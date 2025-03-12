use std::{convert::Infallible, str::FromStr};

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
    Delay(Delay),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnceTimingPart(String);

impl TryInto<OnceTiming> for &[OnceTimingPart] {
    type Error = nom::Err<String>;

    fn try_into(self) -> Result<OnceTiming, Self::Error> {
        let str = self
            .as_ref()
            .iter()
            .map(|part| part.0.as_ref())
            .fold(String::new(), |a, b| a + b + " ");
        OnceTiming::from_str(str.trim())
    }
}

impl FromStr for OnceTimingPart {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(OnceTimingPart(s.to_owned()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RepeatTiming {
    Exact(RepeatExact),
    Delay(Delay),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatTimingPart(String);

impl TryInto<RepeatTiming> for &[RepeatTimingPart] {
    type Error = nom::Err<String>;

    fn try_into(self) -> Result<RepeatTiming, Self::Error> {
        let str = self
            .as_ref()
            .iter()
            .map(|part| part.0.as_ref())
            .fold(String::new(), |a, b| a + b + " ");
        RepeatTiming::from_str(str.trim())
    }
}

impl FromStr for RepeatTimingPart {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RepeatTimingPart(s.to_owned()))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Delay {
    Seconds(i64),
    Days(i64),
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
        // TODO after fixing below TODOs: does this affect when I display notifications or is this
        // broken already? Either way I need to ensure that notifications are handled correctly.
        match self {
            RepeatExact::DayOfMonth { day, time } => Ok(NaiveDateTime::new(
                // TODO #13 I need to ensure that this is in the future. Meaning that if day is less
                // than now.day() this needs to be incremented by a month (properly handling dec =>
                // jan)
                NaiveDate::from_ymd_opt(now.year(), now.month(), *day as u32)
                    .ok_or(TimeError::InvalidDay(*day))?,
                time.unwrap_or(midnight),
            )),
            RepeatExact::DayOfWeek { day, time } => {
                // TODO ensure that this is in the future (see TODO above)
                let today = now.date();
                let week_start = monday_same_week(&today);
                let day = week_start + Days::new((*day).into());
                Ok(NaiveDateTime::new(day, time.clone().unwrap_or(midnight)))
            }

            RepeatExact::Daily { time } => {
                // TODO ensure that this is in the future (see TODO above)
                let today = now.date();
                Ok(NaiveDateTime::new(today, time.unwrap_or(midnight)))
            }
        }
    }
}
