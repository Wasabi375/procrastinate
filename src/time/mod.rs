use std::{convert::Infallible, str::FromStr};

use chrono::{Datelike, Days, Months, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use nom::{branch::alt, IResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::nom_ext::consume_all;

use self::parsing::{parse_duration, parse_rough_instant};

pub mod parsing;

const MIDNIGHT: NaiveTime = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

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
    InvalidDay(u32),
    #[error("{0} is not a valid month")]
    InvalidMonth(u32),
}

impl RoughInstant {
    pub fn notification_date(
        &self,
        last_timestamp: NaiveDateTime,
    ) -> Result<NaiveDateTime, TimeError> {
        match self {
            RoughInstant::DayOfMonth { day, time } => {
                day_of_month_after(last_timestamp, *day as u32, *time)
            }
            RoughInstant::DayOfWeek { day, time } => {
                day_of_week_after(last_timestamp, *day as u32, *time)
            }
            RoughInstant::Date { date } => Ok(date.clone()),
            RoughInstant::Month { month } => {
                let same_year = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(last_timestamp.year(), *month as u32, 1)
                        .ok_or(TimeError::InvalidMonth(*month as u32))?,
                    MIDNIGHT,
                );
                if same_year < last_timestamp {
                    Ok(same_year
                        .checked_add_months(Months::new(12))
                        .expect("Date overflow happens sometime in 262,143"))
                } else {
                    Ok(same_year)
                }
            }
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
    pub fn notification_date(
        &self,
        last_timestamp: NaiveDateTime,
    ) -> Result<NaiveDateTime, TimeError> {
        match self {
            RepeatExact::DayOfMonth { day, time } => {
                day_of_month_after(last_timestamp, *day as u32, *time)
            }
            RepeatExact::DayOfWeek { day, time } => {
                day_of_week_after(last_timestamp, *day as u32, *time)
            }
            RepeatExact::Daily { time } => {
                let same_day = NaiveDateTime::new(last_timestamp.date(), time.unwrap_or(MIDNIGHT));
                if same_day < last_timestamp {
                    Ok(same_day
                        .checked_add_days(Days::new(1))
                        .expect("Date overflow happens sometime in 262,143"))
                } else {
                    Ok(same_day)
                }
            }
        }
    }
}

fn monday_same_week(date: &NaiveDate) -> NaiveDate {
    let days_since_mon = date.weekday().days_since(Weekday::Mon);
    *date - Days::new(days_since_mon.into())
}

fn day_of_month(year: i32, month: u32, mut day: u32) -> Result<NaiveDate, TimeError> {
    if day > 31 {
        return Err(TimeError::InvalidDay(day));
    }
    if month > 12 {
        return Err(TimeError::InvalidMonth(month));
    }
    let orig_day = day;
    while day > 0 && day <= 31 {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            return Ok(date);
        }
        day -= 1;
    }
    panic!("Naive Date invalid: {year}-{month}-{orig_day}");
}

fn day_of_month_after(
    last_timestamp: NaiveDateTime,
    day: u32,
    time: Option<NaiveTime>,
) -> Result<NaiveDateTime, TimeError> {
    let date_same_month = NaiveDateTime::new(
        day_of_month(last_timestamp.year(), last_timestamp.month(), day)?,
        time.unwrap_or(MIDNIGHT),
    );
    if date_same_month < last_timestamp {
        let next_month = last_timestamp
            .checked_add_months(Months::new(1))
            .expect("Date overflow happens sometime in 262,143");

        let date_next_month = NaiveDateTime::new(
            day_of_month(next_month.year(), next_month.month(), day)?,
            time.unwrap_or(MIDNIGHT),
        );
        Ok(date_next_month)
    } else {
        Ok(date_same_month)
    }
}

fn day_of_week_after(
    last_timestamp: NaiveDateTime,
    day: u32,
    time: Option<NaiveTime>,
) -> Result<NaiveDateTime, TimeError> {
    let week_start = monday_same_week(&last_timestamp.date());
    let day = week_start + Days::new(day.into());
    let date_same_week = NaiveDateTime::new(day, time.clone().unwrap_or(MIDNIGHT));
    if date_same_week < last_timestamp {
        Ok(date_same_week
            .checked_add_days(Days::new(7))
            .expect("Date overflow happens sometime in 262,143"))
    } else {
        Ok(date_same_week)
    }
}
