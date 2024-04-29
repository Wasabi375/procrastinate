// TODO temp
#![allow(dead_code)]

use chrono::{NaiveDateTime, NaiveTime};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{self, digit1},
    combinator::{complete, fail, map_parser, map_res, opt},
    sequence::preceded,
    IResult,
};
use std::{ops::Add, str::FromStr, time::Duration};

mod nom_ext;

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

#[derive(Clone, Debug)]
enum RoughInstant {
    DayOfMonth { day: u8, time: Option<NaiveTime> },
    DayOfWeek { day: u8, time: Option<NaiveTime> },
    Today { time: NaiveTime },
    Tomorrow { time: Option<NaiveTime> },
    Date { date: NaiveDateTime },
    Month { month: u8 },
}

fn parse_rough_instant(input: &str) -> IResult<&str, RoughInstant> {
    use rough_instant::*;
    alt((
        parse_day_of_month,
        parse_day_of_week,
        parse_today,
        parse_tomorrow,
        parse_date,
        parse_month,
    ))(input)
}

mod rough_instant {
    use crate::{nom_ext::alt_many, parse_digits, parse_time, RoughInstant, DAYS_IN_WEEK, MONTHS};
    use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
    use nom::{
        branch::alt,
        bytes::complete::{tag, tag_no_case},
        character::complete,
        combinator::{fail, opt},
        sequence::{pair, preceded, tuple},
        IResult,
    };

    pub fn parse_day_of_month(input: &str) -> IResult<&str, RoughInstant> {
        let (input, _) = pair(tag("dom"), complete::char(' '))(input)?;

        let (input, day) = parse_digits(input)?;

        if day == 0 || day > 31 {
            fail::<_, RoughInstant, _>(input)?;
        }

        let (input, time) = opt(preceded(complete::char(' '), parse_time))(input)?;

        Ok((input, RoughInstant::DayOfMonth { day, time }))
    }

    pub fn parse_day_of_week(input: &str) -> IResult<&str, RoughInstant> {
        use nom::Parser;
        let (input, day) = alt_many(
            DAYS_IN_WEEK.map(|tag| tag_no_case::<&str, &str, nom::error::Error<&str>>(tag)),
        )
        .parse(input)?;

        let Some(day) = DAYS_IN_WEEK
            .iter()
            .enumerate()
            .find(|(_, it)| **it == day.to_ascii_lowercase())
            .map(|(i, _)| i as u8)
        else {
            fail::<_, RoughInstant, _>(input)?;
            unreachable!();
        };

        let (input, time) = opt(preceded(complete::char(' '), parse_time))(input)?;

        Ok((input, RoughInstant::DayOfWeek { day, time }))
    }

    pub fn parse_today(input: &str) -> IResult<&str, RoughInstant> {
        let (input, _tag) = tag("today")(input)?;

        let (input, time) = preceded(complete::char(' '), parse_time)(input)?;

        Ok((input, RoughInstant::Today { time }))
    }

    pub fn parse_tomorrow(input: &str) -> IResult<&str, RoughInstant> {
        let (input, _tag) = tag("tomorrow")(input)?;

        let (input, time) = opt(preceded(complete::char(' '), parse_time))(input)?;

        Ok((input, RoughInstant::Tomorrow { time }))
    }

    pub fn parse_month(input: &str) -> IResult<&str, RoughInstant> {
        use nom::Parser;
        let (input, month) =
            alt_many(MONTHS.map(|tag| tag_no_case::<&str, &str, nom::error::Error<&str>>(tag)))
                .parse(input)?;

        let Some(month) = MONTHS
            .iter()
            .enumerate()
            .find(|(_, it)| **it == month.to_ascii_lowercase())
            .map(|(i, _)| i as u8)
        else {
            fail::<_, RoughInstant, _>(input)?;
            unreachable!();
        };

        Ok((input, RoughInstant::Month { month }))
    }

    pub fn parse_date(input: &str) -> IResult<&str, RoughInstant> {
        let (input, date) = opt(alt((parse_ymd, parse_day_month)))(input)?;

        let (input, time) = opt(parse_time)(input)?;

        if date.is_none() && time.is_none() {
            fail::<_, RoughInstant, _>(input)?;
        }

        let date = date.unwrap_or_else(|| Local::now().date_naive());
        let time = time.unwrap_or_else(|| Local::now().time());

        let datetime = NaiveDateTime::new(date, time);

        Ok((input, RoughInstant::Date { date: datetime }))
    }

    fn parse_ymd(input: &str) -> IResult<&str, NaiveDate> {
        let dash = complete::char::<&str, nom::error::Error<&str>>('-');

        let (input, (year, _, month, _, day)) = tuple((
            parse_digits::<i32>,
            &dash,
            parse_digits::<u32>,
            &dash,
            parse_digits::<u32>,
        ))(input)?;

        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => Ok((input, date)),
            None => fail(input),
        }
    }

    fn parse_day_month(input: &str) -> IResult<&str, NaiveDate> {
        let dash = complete::char::<&str, nom::error::Error<&str>>('-');

        let (input, (day, _, month)) =
            tuple((parse_digits::<u32>, dash, parse_digits::<u32>))(input)?;

        let year = Local::now().year();
        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => Ok((input, date)),
            None => fail(input),
        }
    }
}

impl FromStr for RoughInstant {
    type Err = nom::Err<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match complete(parse_rough_instant)(s) {
            Ok((_, instant)) => Ok(instant),
            Err(error) => match error {
                nom::Err::Incomplete(err) => Err(nom::Err::Incomplete(err)),
                nom::Err::Error(err) => Err(nom::Err::Error(err.to_string())),
                nom::Err::Failure(err) => Err(nom::Err::Failure(err.to_string())),
            },
        }
    }
}

#[derive(Clone, Debug)]
enum RepeatExact {
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

fn parse_repeat_exact(input: &str) -> IResult<&str, RepeatExact> {
    use repeat_exact::*;
    alt((parse_day_of_month, parse_day_of_week, parse_daily))(input)
}

const DAYS_IN_WEEK: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

const MONTHS: [&str; 12] = [
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

mod repeat_exact {
    use nom::{
        bytes::complete::{tag, tag_no_case},
        character::complete,
        combinator::{fail, opt},
        sequence::{pair, preceded},
        IResult,
    };

    use crate::{nom_ext::alt_many, parse_digits, parse_time, RepeatExact, DAYS_IN_WEEK};

    /// parse [RepeatExact::Daily]
    ///
    /// Valid: `daily[ <time-of-day>]`
    pub fn parse_daily(input: &str) -> IResult<&str, RepeatExact> {
        let (input, _) = pair(tag("daily"), complete::char(' '))(input)?;

        let (input, time) = opt(preceded(complete::char(' '), parse_time))(input)?;

        Ok((input, RepeatExact::Daily { time }))
    }

    /// parse [RepeatExact::DayOfMonth].
    ///
    /// Valid: `monthly <day> [ <time-of-day>]`
    /// `<day>`: First, second, of the month as 1, 2, etc
    pub fn parse_day_of_month(input: &str) -> IResult<&str, RepeatExact> {
        let (input, _) = pair(tag("monthly"), complete::char(' '))(input)?;

        let (input, day) = parse_digits(input)?;

        if day == 0 || day > 31 {
            fail::<_, RepeatExact, _>(input)?;
        }

        let (input, time) = opt(preceded(complete::char(' '), parse_time))(input)?;

        Ok((input, RepeatExact::DayOfMonth { day, time }))
    }

    /// parse [RepeatExact::DayOfWeek].
    ///
    /// Valid: `<day-of-week>[ <time-of-day>]`
    pub fn parse_day_of_week(input: &str) -> IResult<&str, RepeatExact> {
        use nom::Parser;
        let (input, day) = alt_many(
            DAYS_IN_WEEK.map(|tag| tag_no_case::<&str, &str, nom::error::Error<&str>>(tag)),
        )
        .parse(input)?;

        let Some(day) = DAYS_IN_WEEK
            .iter()
            .enumerate()
            .find(|(_, it)| **it == day.to_ascii_lowercase())
            .map(|(i, _)| i as u8)
        else {
            fail::<_, RepeatExact, _>(input)?;
            unreachable!();
        };

        let (input, time) = opt(preceded(complete::char(' '), parse_time))(input)?;

        Ok((input, RepeatExact::DayOfWeek { day, time }))
    }
}

/// Parse multiple ascii digits into I
fn parse_digits<I: FromStr>(input: &str) -> IResult<&str, I> {
    map_res(digit1, |s: &str| s.parse::<I>())(input)
}

/// Parses a time in `hh:mm[:ss]` format
fn parse_time(input: &str) -> IResult<&str, NaiveTime> {
    let (input, hour) = map_parser(
        take_while_m_n(1, 2, |c: char| c.is_ascii_digit()),
        parse_digits::<u32>,
    )(input)?;

    let (input, _c) = complete::char(':')(input)?;

    let (input, min) = map_parser(
        take_while_m_n(1, 2, |c: char| c.is_ascii_digit()),
        parse_digits::<u32>,
    )(input)?;

    let (input, sec) = opt(preceded(
        complete::char(':'),
        map_parser(
            take_while_m_n(1, 2, |c: char| c.is_ascii_digit()),
            parse_digits::<u32>,
        ),
    ))(input)?;

    match NaiveTime::from_hms_opt(hour, min, sec.unwrap_or(0)) {
        Some(time) => Ok((input, time)),
        None => fail(input),
    }
}

impl FromStr for RepeatExact {
    type Err = nom::Err<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match complete(parse_repeat_exact)(s) {
            Ok((_, repeat)) => Ok(repeat),
            Err(error) => match error {
                nom::Err::Incomplete(err) => Err(nom::Err::Incomplete(err)),
                nom::Err::Error(err) => Err(nom::Err::Error(err.to_string())),
                nom::Err::Failure(err) => Err(nom::Err::Failure(err.to_string())),
            },
        }
    }
}

#[derive(Clone, Debug)]
struct Delay(Duration);

const SECONDS_IN_HOUR: u64 = 60 * 60;
const SECONDS_IN_DAY: u64 = SECONDS_IN_HOUR * 24;
const SECONDS_IN_WEEK: u64 = SECONDS_IN_DAY * 7;
const SECONDS_IN_MONTH: u64 = SECONDS_IN_DAY * 30;
const SECONDS_IN_YEAR: u64 = SECONDS_IN_DAY * 365;

macro_rules! duration_parser {
    ($fn_name:ident, $long:literal, $short:literal, $mul:expr) => {
        fn $fn_name(input: &str) -> IResult<&str, Duration> {
            let (input, count) = parse_digits::<u64>(input)?;

            // TODO do I want to ignore white space before long/short?
            let (input, _tag) = alt((tag($long), tag($short)))(input)?;

            Ok((input, Duration::from_secs(count * $mul)))
        }
    };
}

duration_parser!(parse_seconds, "sec", "s", 1);
duration_parser!(parse_minutes, "min", "m", 60);
duration_parser!(parse_hours, "hour", "h", SECONDS_IN_HOUR);
duration_parser!(parse_days, "days", "d", SECONDS_IN_DAY);
duration_parser!(parse_weeks, "weeks", "w", SECONDS_IN_WEEK);
duration_parser!(parse_months, "months", "M", SECONDS_IN_MONTH);
duration_parser!(parse_year, "year", "y", SECONDS_IN_YEAR);

fn reduce<T, F>(a: Option<T>, b: Option<T>, f: F) -> Option<T>
where
    T: std::fmt::Debug,
    F: FnOnce(T, T) -> T,
{
    match a {
        Some(a) => match b {
            Some(b) => Some(f(a, b)),
            None => Some(a),
        },
        None => b,
    }
}

fn parse_duration(input: &str) -> IResult<&str, Duration> {
    let mut result = None;
    let (input, duration) = opt(parse_year)(input)?;
    result = reduce(result, duration, Duration::add);
    let (input, duration) = opt(parse_months)(input)?;
    result = reduce(result, duration, Duration::add);
    let (input, duration) = opt(parse_weeks)(input)?;
    result = reduce(result, duration, Duration::add);
    let (input, duration) = opt(parse_days)(input)?;
    result = reduce(result, duration, Duration::add);
    let (input, duration) = opt(parse_hours)(input)?;
    result = reduce(result, duration, Duration::add);
    let (input, duration) = opt(parse_minutes)(input)?;
    result = reduce(result, duration, Duration::add);
    let (input, duration) = opt(parse_seconds)(input)?;
    result = reduce(result, duration, Duration::add);

    match result {
        Some(r) => Ok((input, r)),
        None => fail(input),
    }
}

impl FromStr for Delay {
    type Err = nom::Err<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match complete(parse_duration)(s) {
            Ok((_, duration)) => Ok(Delay(duration)),
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
