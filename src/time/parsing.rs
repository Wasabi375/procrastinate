use chrono::NaiveTime;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{self, digit1},
    combinator::{fail, map_parser, map_res, opt},
    sequence::preceded,
    IResult,
};
use std::{ops::Add, str::FromStr, time::Duration};

use super::{
    Delay, RepeatExact, RoughInstant, SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MONTH,
    SECONDS_IN_WEEK, SECONDS_IN_YEAR,
};

/// Parse multiple ascii digits into I
fn parse_digits<I>(input: &str) -> IResult<&str, I>
where
    I: FromStr,
{
    map_res(digit1, |s: &str| s.parse::<I>())(input)
}

/// Parses a time in `hh:mm[:ss]` format
pub fn parse_time(input: &str) -> IResult<&str, NaiveTime> {
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

pub fn parse_duration(input: &str) -> IResult<&str, Delay> {
    let mut seconds = false;
    let mut result = None;

    let (input, duration) = opt(parse_year)(input)?;
    let (input, _) = opt(complete::char(' '))(input)?;
    result = reduce(result, duration, Duration::add);

    let (input, duration) = opt(parse_months)(input)?;
    let (input, _) = opt(complete::char(' '))(input)?;
    result = reduce(result, duration, Duration::add);

    let (input, duration) = opt(parse_weeks)(input)?;
    let (input, _) = opt(complete::char(' '))(input)?;
    result = reduce(result, duration, Duration::add);

    let (input, duration) = opt(parse_days)(input)?;
    let (input, _) = opt(complete::char(' '))(input)?;
    result = reduce(result, duration, Duration::add);

    let (input, duration) = opt(parse_hours)(input)?;
    let (input, _) = opt(complete::char(' '))(input)?;
    seconds |= duration.is_some();
    result = reduce(result, duration, Duration::add);

    let (input, duration) = opt(parse_minutes)(input)?;
    let (input, _) = opt(complete::char(' '))(input)?;
    seconds |= duration.is_some();
    result = reduce(result, duration, Duration::add);

    let (input, duration) = opt(parse_seconds)(input)?;
    seconds |= duration.is_some();
    result = reduce(result, duration, Duration::add);

    match (result, seconds) {
        (Some(duration), true) => Ok((
            input,
            Delay::Seconds(
                duration
                    .as_secs()
                    .try_into()
                    .expect("seconds value must fit within i64"),
            ),
        )),
        (Some(duration), false) => Ok((
            input,
            Delay::Days(
                (duration.as_secs() / SECONDS_IN_DAY)
                    .try_into()
                    .expect("days value must fit within i64"),
            ),
        )),
        (None, _) => fail(input),
    }
}

pub fn parse_rough_instant(input: &str) -> IResult<&str, RoughInstant> {
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
    use crate::{
        nom_ext::alt_many,
        time::{RoughInstant, DAYS_IN_WEEK, MONTHS},
    };
    use chrono::{Datelike, Days, Local, NaiveDate, NaiveDateTime, NaiveTime};
    use nom::{
        branch::alt,
        bytes::complete::{tag, tag_no_case},
        character::complete,
        combinator::{fail, opt},
        sequence::{pair, preceded, tuple},
        IResult,
    };

    use super::{parse_digits, parse_time};

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

        let today = Local::now().date_naive();
        let date = NaiveDateTime::new(today, time);

        Ok((input, RoughInstant::Date { date }))
    }

    pub fn parse_tomorrow(input: &str) -> IResult<&str, RoughInstant> {
        let (input, _tag) = tag("tomorrow")(input)?;

        let (input, time) = opt(preceded(complete::char(' '), parse_time))(input)?;

        let today = Local::now().date_naive();
        let tomorrow = today + Days::new(1);
        let date = NaiveDateTime::new(
            tomorrow,
            time.unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
        );

        Ok((input, RoughInstant::Date { date }))
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

        let (input, time) = opt(preceded(complete::char(' '), parse_time))(input)?;

        if date.is_none() && time.is_none() {
            fail::<_, RoughInstant, _>(input)?;
        }

        let date = date.unwrap_or_else(|| Local::now().date_naive());
        let time = time.unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

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

    #[cfg(test)]
    mod test {
        use chrono::{Datelike, Local, NaiveDate, NaiveTime};

        use super::*;

        fn year() -> i32 {
            Local::now().year()
        }

        #[test]
        fn test_parse_day_of_month() {
            for i in 1..=31 {
                assert_eq!(
                    parse_day_of_month(&format!("dom {i}")),
                    Ok((
                        "",
                        RoughInstant::DayOfMonth {
                            day: i as u8,
                            time: None
                        }
                    ))
                );
                assert_eq!(
                    parse_day_of_month(&format!("dom {i} 22:0")),
                    Ok((
                        "",
                        RoughInstant::DayOfMonth {
                            day: i as u8,
                            time: NaiveTime::from_hms_opt(22, 0, 0)
                        }
                    ))
                );
            }
        }

        #[test]
        fn test_parse_day_of_week() {
            for (i, day) in DAYS_IN_WEEK.iter().enumerate() {
                assert_eq!(
                    parse_day_of_week(day),
                    Ok((
                        "",
                        RoughInstant::DayOfWeek {
                            day: i as u8,
                            time: None
                        }
                    ))
                );
                assert_eq!(
                    parse_day_of_week(&format!("{day} 3:11:33")),
                    Ok((
                        "",
                        RoughInstant::DayOfWeek {
                            day: i as u8,
                            time: NaiveTime::from_hms_opt(3, 11, 33)
                        }
                    ))
                );
            }
        }

        #[test]
        fn test_parse_today() {
            assert!(parse_today("today").is_err());
            let today = Local::now().date_naive();
            assert_eq!(
                parse_today("today 07:42"),
                Ok((
                    "",
                    RoughInstant::Date {
                        date: NaiveDateTime::new(today, NaiveTime::from_hms_opt(7, 42, 0).unwrap())
                    }
                ))
            );
        }

        #[test]
        fn test_parse_tomorrow() {
            let today = Local::now().date_naive();
            let tomorrow = today + Days::new(1);
            assert_eq!(
                parse_tomorrow("tomorrow"),
                Ok((
                    "",
                    RoughInstant::Date {
                        date: NaiveDateTime::new(
                            tomorrow,
                            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                        )
                    }
                ))
            );
            assert_eq!(
                parse_tomorrow("tomorrow 07:42"),
                Ok((
                    "",
                    RoughInstant::Date {
                        date: NaiveDateTime::new(
                            tomorrow,
                            NaiveTime::from_hms_opt(7, 42, 0).unwrap()
                        )
                    }
                ))
            );
        }

        #[test]
        fn test_parse_month() {
            for (i, month) in MONTHS.iter().enumerate() {
                assert_eq!(
                    parse_month(month),
                    Ok(("", RoughInstant::Month { month: i as u8 }))
                );
                let mut cap_month = String::with_capacity(month.len());
                cap_month.push_str(&month.chars().next().unwrap().to_uppercase().to_string());
                cap_month.push_str(&month[1..]);
                assert_eq!(
                    parse_month(&cap_month),
                    Ok(("", RoughInstant::Month { month: i as u8 }))
                );
            }
        }

        #[test]
        fn test_parse_date() {
            assert_eq!(
                parse_date("14-12"),
                Ok((
                    "",
                    RoughInstant::Date {
                        date: NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(year(), 12, 14).unwrap(),
                            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                        )
                    }
                ))
            );
            assert_eq!(
                parse_date("2024-11-25"),
                Ok((
                    "",
                    RoughInstant::Date {
                        date: NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2024, 11, 25).unwrap(),
                            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                        )
                    }
                ))
            );
            assert_eq!(
                parse_date("14-12 9:13"),
                Ok((
                    "",
                    RoughInstant::Date {
                        date: NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(year(), 12, 14).unwrap(),
                            NaiveTime::from_hms_opt(9, 13, 0).unwrap()
                        )
                    }
                ))
            );
            assert_eq!(
                parse_date("2024-11-25 00:00"),
                Ok((
                    "",
                    RoughInstant::Date {
                        date: NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(2024, 11, 25).unwrap(),
                            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                        )
                    }
                ))
            );
        }

        #[test]
        fn test_parse_day_month() {
            assert_eq!(
                parse_day_month("14-12"),
                Ok(("", NaiveDate::from_ymd_opt(year(), 12, 14).unwrap()))
            );
        }

        #[test]
        fn test_parse_ymd() {
            assert_eq!(
                parse_ymd("2024-11-25"),
                Ok(("", NaiveDate::from_ymd_opt(2024, 11, 25).unwrap()))
            );
        }
    }
}

pub fn parse_repeat_exact(input: &str) -> IResult<&str, RepeatExact> {
    use repeat_exact::*;
    alt((parse_day_of_month, parse_day_of_week, parse_daily))(input)
}

mod repeat_exact {
    use nom::{
        bytes::complete::{tag, tag_no_case},
        character::complete,
        combinator::{fail, opt},
        sequence::{pair, preceded},
        IResult,
    };

    use crate::{
        nom_ext::alt_many,
        time::{RepeatExact, DAYS_IN_WEEK},
    };

    use super::{parse_digits, parse_time};

    /// parse [RepeatExact::Daily]
    ///
    /// Valid: `daily[ <time-of-day>]`
    pub fn parse_daily(input: &str) -> IResult<&str, RepeatExact> {
        let (input, _) = tag("daily")(input)?;

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

    #[cfg(test)]
    mod test {
        use chrono::NaiveTime;

        use super::*;
        use crate::time::DAYS_IN_WEEK;

        #[test]
        fn test_parse_daily() {
            assert_eq!(
                parse_daily("daily"),
                Ok(("", RepeatExact::Daily { time: None })),
                "daily"
            );
            assert_eq!(
                parse_daily("daily rest"),
                Ok((" rest", RepeatExact::Daily { time: None })),
                "daily rest"
            );
            assert_eq!(
                parse_daily("daily 14:59"),
                Ok((
                    "",
                    RepeatExact::Daily {
                        time: NaiveTime::from_hms_opt(14, 59, 0)
                    }
                )),
                "daily 14:59"
            );
        }

        #[test]
        fn test_parse_day_of_week() {
            for (i, day) in DAYS_IN_WEEK.iter().enumerate() {
                assert_eq!(
                    parse_day_of_week(day),
                    Ok((
                        "",
                        RepeatExact::DayOfWeek {
                            day: i as u8,
                            time: None
                        }
                    ))
                );
                let mut cap_day = String::with_capacity(day.len());
                cap_day.push_str(&day.chars().next().unwrap().to_uppercase().to_string());
                cap_day.push_str(&day[1..]);
                assert_eq!(
                    parse_day_of_week(&cap_day),
                    Ok((
                        "",
                        RepeatExact::DayOfWeek {
                            day: i as u8,
                            time: None
                        }
                    ))
                );
                assert_eq!(
                    parse_day_of_week(&format!("{day} 15:27")),
                    Ok((
                        "",
                        RepeatExact::DayOfWeek {
                            day: i as u8,
                            time: NaiveTime::from_hms_opt(15, 27, 0)
                        }
                    ))
                );
                assert_eq!(
                    parse_day_of_week(&format!("{day} 15:27 rest")),
                    Ok((
                        " rest",
                        RepeatExact::DayOfWeek {
                            day: i as u8,
                            time: NaiveTime::from_hms_opt(15, 27, 0)
                        }
                    ))
                );
            }
        }

        #[test]
        fn test_parse_day_of_month() {
            assert_eq!(
                parse_day_of_month("monthly 1"),
                Ok(("", RepeatExact::DayOfMonth { day: 1, time: None }))
            );
            assert_eq!(
                parse_day_of_month("monthly 31"),
                Ok((
                    "",
                    RepeatExact::DayOfMonth {
                        day: 31,
                        time: None
                    }
                ))
            );
            assert!(parse_day_of_month("monthly 0").is_err());
            assert!(parse_day_of_month("monthly 32").is_err());
            assert_eq!(
                parse_day_of_month("monthly 1 12:31"),
                Ok((
                    "",
                    RepeatExact::DayOfMonth {
                        day: 1,
                        time: NaiveTime::from_hms_opt(12, 31, 0)
                    }
                ))
            );
            assert_eq!(
                parse_day_of_month("monthly 1 12:31:15"),
                Ok((
                    "",
                    RepeatExact::DayOfMonth {
                        day: 1,
                        time: NaiveTime::from_hms_opt(12, 31, 15)
                    }
                ))
            );
            assert_eq!(
                parse_day_of_month("monthly 1rest"),
                Ok(("rest", RepeatExact::DayOfMonth { day: 1, time: None }))
            );
            assert_eq!(
                parse_day_of_month("monthly 1 12:31rest"),
                Ok((
                    "rest",
                    RepeatExact::DayOfMonth {
                        day: 1,
                        time: NaiveTime::from_hms_opt(12, 31, 0)
                    }
                ))
            );
            assert_eq!(
                parse_day_of_month("monthly 1 12:31:15rest"),
                Ok((
                    "rest",
                    RepeatExact::DayOfMonth {
                        day: 1,
                        time: NaiveTime::from_hms_opt(12, 31, 15)
                    }
                ))
            );
        }
    }
}

#[cfg(test)]
mod test {
    use crate::nom_ext::consume_all;

    use super::*;

    // hh:mm[:ss]
    #[test]
    fn test_parse_time() {
        assert_eq!(
            parse_time("0:0"),
            Ok(("", NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
        );
        assert_eq!(
            parse_time("0:0:0"),
            Ok(("", NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
        );
        assert_eq!(
            parse_time("1:2"),
            Ok(("", NaiveTime::from_hms_opt(1, 2, 0).unwrap()))
        );
        assert_eq!(
            parse_time("1:2:3"),
            Ok(("", NaiveTime::from_hms_opt(1, 2, 3).unwrap()))
        );
        assert_eq!(
            parse_time("10:11"),
            Ok(("", NaiveTime::from_hms_opt(10, 11, 0).unwrap()))
        );
        assert_eq!(
            parse_time("10:11:12"),
            Ok(("", NaiveTime::from_hms_opt(10, 11, 12).unwrap()))
        );
        assert!(parse_time("25:0").is_err());
        assert!(parse_time("12:61").is_err());
        assert!(parse_time("12:42:61").is_err());
    }

    macro_rules! duration_parser_test {
        ($test_name:ident, $fn_name:ident, $long:literal, $short:literal, $mul:expr) => {
            #[test]
            fn $test_name() {
                assert_eq!(
                    $fn_name(&format!("12{}", $long)),
                    Ok(("", Duration::from_secs(12 * $mul)))
                );
                assert_eq!(
                    $fn_name(&format!("24{}", $short)),
                    Ok(("", Duration::from_secs(24 * $mul)))
                );
            }
        };
    }

    duration_parser_test!(test_parse_seconds, parse_seconds, "sec", "s", 1);
    duration_parser_test!(test_parse_minutes, parse_minutes, "min", "m", 60);
    duration_parser_test!(test_parse_hours, parse_hours, "hour", "h", SECONDS_IN_HOUR);
    duration_parser_test!(test_parse_days, parse_days, "days", "d", SECONDS_IN_DAY);
    duration_parser_test!(test_parse_weeks, parse_weeks, "weeks", "w", SECONDS_IN_WEEK);
    duration_parser_test!(
        test_parse_months,
        parse_months,
        "months",
        "M",
        SECONDS_IN_MONTH
    );
    duration_parser_test!(test_parse_year, parse_year, "year", "y", SECONDS_IN_YEAR);

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("12sec"), Ok(("", Delay::Seconds(12))));
        assert_eq!(parse_duration("12s"), Ok(("", Delay::Seconds(12))));
        assert_eq!(parse_duration("12m"), Ok(("", Delay::Seconds(12 * 60))));
        assert_eq!(
            parse_duration("12h"),
            Ok(("", Delay::Seconds(12 * SECONDS_IN_HOUR as i64)))
        );
        assert_eq!(parse_duration("12d"), Ok(("", Delay::Days(12))));
        assert_eq!(parse_duration("12w"), Ok(("", Delay::Days(12 * 7))));
        assert_eq!(parse_duration("12M"), Ok(("", Delay::Days(12 * 30))));
        assert_eq!(parse_duration("12y"), Ok(("", Delay::Days(12 * 365))));

        assert_eq!(
            parse_duration("3d 5s"),
            Ok(("", Delay::Seconds(3 * SECONDS_IN_DAY as i64 + 5)))
        );
        assert_eq!(parse_duration("2w 3d"), Ok(("", Delay::Days(2 * 7 + 3))));
        assert!(parse_duration("5").is_err());
        assert!(consume_all(parse_duration)("5d 3w").is_err());
    }

    #[test]
    fn test_parse_duration_multiday_hours() {
        assert_eq!(
            parse_duration("24h"),
            Ok(("", Delay::Seconds(24 * SECONDS_IN_HOUR as i64)))
        );
        assert_eq!(
            parse_duration("48h"),
            Ok(("", Delay::Seconds(48 * SECONDS_IN_HOUR as i64)))
        );
    }
}
