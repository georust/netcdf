//! Time handling according to CF conventions
#![allow(missing_docs)]

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{i32, i8, one_of, space1, u32, u8},
    combinator::{all_consuming, map, opt, peek, value},
    number::complete::double,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};

/// Parsing error
#[derive(Debug)]
pub struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}

/// Base duration between time points
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Duration {
    Years,
    Months,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Time {
    pub hour: i32,
    pub minute: u32,
    pub second: u32,
    pub millisecond: u32,
    pub microsecond: u32,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
    pub tz: Tz,
}

fn duration(input: &str) -> IResult<&str, Duration> {
    #[rustfmt::skip]
    let years = value(
        Duration::Years,
        alt((
            tag("common_years"),
            tag("common_year")
        )),
    );
    #[rustfmt::skip]
    let months = value(
        Duration::Months,
        alt((
            tag("months"),
            tag("month")
        ))
    );
    #[rustfmt::skip]
    let days = value(
        Duration::Days,
        alt((
            tag("days"),
            tag("day"),
            tag("d")
        ))
    );
    #[rustfmt::skip]
    let hours = value(
        Duration::Hours,
        alt((
            tag("hours"),
            tag("hour"),
            tag("hrs"),
            tag("hr"),
            tag("h")
        )),
    );
    #[rustfmt::skip]
    let minutes = value(
        Duration::Minutes,
        alt((
            tag("minutes"),
            tag("minute"),
            tag("mins"),
            tag("min")
        )),
    );
    let seconds = value(
        Duration::Seconds,
        alt((
            tag("seconds"),
            tag("second"),
            tag("secs"),
            tag("sec"),
            tag("s"),
        )),
    );
    let milliseconds = value(
        Duration::Milliseconds,
        alt((
            tag("milliseconds"),
            tag("millisecond"),
            tag("millisecs"),
            tag("millisec"),
            tag("msecs"),
            tag("msec"),
            tag("ms"),
        )),
    );
    let microseconds = value(
        Duration::Microseconds,
        alt((
            tag("microseconds"),
            tag("microsecond"),
            tag("microsecs"),
            tag("microsec"),
        )),
    );

    alt((
        years,
        months,
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
    ))(input)
}

fn date(input: &str) -> IResult<&str, Date> {
    let ymd = map(
        tuple((u32, tag("-"), u32, tag("-"), u32)),
        |(year, _, month, _, day)| Date { year, month, day },
    );

    alt((ymd,))(input)
}

fn time(input: &str) -> IResult<&str, Time> {
    let hms = map(
        tuple((i32, tag(":"), u32, tag(":"), double)),
        |(hour, _, minute, _, second)| {
            let (second, rest) = (second.trunc(), second.fract());
            let millisecond = rest * 1000.0;
            let (millisecond, rest) = (millisecond.trunc(), millisecond.fract());
            let microsecond = rest * 1000.0;

            Time {
                hour,
                minute,
                second: second as _,
                millisecond: millisecond as _,
                microsecond: microsecond as _,
            }
        },
    );

    let hm = map(separated_pair(i32, tag(":"), u32), |(hour, minute)| Time {
        hour,
        minute,
        ..Time::default()
    });

    alt((hms, hm))(input)
}

fn timezone(input: &str) -> IResult<&str, Tz> {
    println!("{input}");
    let hm = map(
        preceded(opt(tag("+")), separated_pair(i8, tag(":"), u8)),
        |(hour, minute)| Tz {
            hour: hour,
            minute: minute,
        },
    );
    let z = value(Tz::default(), tag("Z"));
    let utc = value(Tz::default(), tag("UTC"));
    alt((hm, z, utc))(input)
}

fn datetime(input: &str) -> IResult<&str, DateTime> {
    fn space1_or_t(input: &str) -> IResult<&str, ()> {
        alt((value((), space1), value((), tag("T"))))(input)
    }
    let tz = map(
        separated_pair(separated_pair(date, space1_or_t, time), space1, timezone),
        |((date, time), tz)| DateTime { date, time, tz },
    );

    let no_tz = map(separated_pair(date, space1_or_t, time), |(date, time)| {
        DateTime {
            date,
            time,
            ..DateTime::default()
        }
    });

    let date_with_tz = map(separated_pair(date, space1, timezone), |(date, tz)| {
        DateTime {
            date,
            tz,
            ..DateTime::default()
        }
    });

    let date_time_no_space_tz = map(
        separated_pair(
            separated_pair(date, space1_or_t, time),
            peek(one_of("+-Z")),
            timezone,
        ),
        |((date, time), tz)| DateTime { date, time, tz },
    );

    let only_date = map(date, |date| DateTime {
        date,
        ..DateTime::default()
    });

    alt((tz, date_time_no_space_tz, no_tz, date_with_tz, only_date))(input)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Tz {
    hour: i8,
    minute: u8,
}

/// Parse a CF compatible string into two components
pub fn cf_parser(input: &str) -> Result<(Duration, DateTime), ParseError> {
    let since = tuple((space1, tag("since"), space1));
    all_consuming(separated_pair(duration, since, datetime))(input)
        .map(|(_, o)| o)
        .map_err(|e| ParseError(format!("{}", e)))
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(input: &str) {
        println!("{:?}", cf_parser(input).unwrap())
    }

    #[test]
    fn cf_conventions_document() {
        parse("days since 1990-1-1 0:0:0");
        parse("seconds since 1992-10-8 15:15:42.5 -6:00");
        parse("days since 1-7-15 0:0:0");
        parse("days since 1-1-1 0:0:0");
    }

    #[test]
    fn cftime_py_setup() {
        parse("hours since 0001-01-01 00:00:00");
        parse("hours since 0001-01-01 00:00:00");
        parse("hours since 0001-01-01 00:00:00 -06:00");
        parse("seconds since 0001-01-01 00:00:00");
        parse("days since 1600-02-28 00:00:00");
        parse("days since 1600-02-29 00:00:00");
        parse("days since 1600-02-30 00:00:00");
        parse("hours since 1000-01-01 00:00:00");
        parse("seconds since 1970-01-01T00:00:00Z");
        parse("days since  850-01-01 00:00:00");
        parse("hours since 0001-01-01 00:00:00");
        parse("days since 1600-02-28 00:00:00");
    }

    #[test]
    fn cftime_py_tz_naive() {
        let d_check = ["1582-10-15 00:00:00", "1582-10-15 12:00:00"];
        for d in d_check {
            parse(&format!("day since {}", d));
        }
    }

    #[test]
    fn cftime_py() {
        parse("days since 1000-01-01");
        parse("seconds since 1970-01-01T00:00:00Z");
        parse("hours since 2013-12-12T12:00:00");
        parse("hours since 1682-10-15 -07:00");
        parse("hours since 1682-10-15 -07:00:12");
        parse("hours since 1682-10-15T-07:00:12");
        parse("hours since 1682-10-15 -07:00 UTC");
        parse("hours since 2000-01-01 22:30+04:00");
        parse("hours since 2000-01-01 11:30-07:00");
        parse("hours since 2000-01-01 15:00-03:30");
    }

    #[test]
    fn etc() {
        parse("seconds since 1992-10-8 15:15:42.5Z");
        parse("seconds since 1992-10-8 15:15:42Z");
    }
}
