use crate::calendars::Calendars;
use crate::durations::CFDuration;
use crate::time::Time;
use crate::tz::Tz;

use crate::datetime_360day::{Date360Day, DateTime360Day};
use crate::datetime_all_leap::{DateAllLeap, DateTimeAllLeap};
use crate::datetime_julian::{DateJulian, DateTimeJulian};
use crate::datetime_no_leap::{DateNoLeap, DateTimeNoLeap};
use crate::datetime_prolecpticgregorian::{DateProlepticGregorian, DateTimeProlepticGregorian};
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

fn duration<'a>(input: &'a str, calendar: &'a Calendars) -> IResult<&'a str, CFDuration> {
    #[rustfmt::skip]
    let years = value(
        CFDuration::years(1, *calendar),
        alt((
            tag("common_years"),
            tag("common_year")
        )),
    );
    #[rustfmt::skip]
    let months = value(
        CFDuration::months(1, *calendar),
        alt((
            tag("months"),
            tag("month")
        ))
    );
    #[rustfmt::skip]
    let days = value(
        CFDuration::days(1, *calendar),
        alt((
            tag("days"),
            tag("day"),
            tag("d")
        ))
    );
    #[rustfmt::skip]
    let hours = value(
        CFDuration::hours(1,*calendar),
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
        CFDuration::minutes(1,*calendar),
        alt((
            tag("minutes"),
            tag("minute"),
            tag("mins"),
            tag("min")
        )),
    );
    let seconds = value(
        CFDuration::seconds(1, *calendar),
        alt((
            tag("seconds"),
            tag("second"),
            tag("secs"),
            tag("sec"),
            tag("s"),
        )),
    );
    let milliseconds = value(
        CFDuration::milliseconds(1, *calendar),
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
        CFDuration::microseconds(1, *calendar),
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

/// macro created to not pass two argument to the function as it is a requirement
/// for nom::separated_pair

fn date<'a>(input: &'a str, calendar: &'a Calendars) -> IResult<&'a str, CFDates> {
    let cf_date_factory = CFDateFactory {
        calendar: *calendar,
    };
    let ymd = map(
        tuple((i32, tag("-"), u32, tag("-"), u32)),
        |(year, _, month, _, day)| cf_date_factory.build(year, month, day).unwrap(),
    );

    let x = alt((ymd,))(input);
    x
}
fn time(input: &str) -> IResult<&str, Time> {
    let hms = map(
        tuple((i32, tag(":"), u32, tag(":"), double)),
        |(hour, _, minute, _, second)| {
            let (second, rest) = (second.trunc(), second.fract());
            let nanosecond = rest * 1e9;

            Time {
                hour: hour as u32,
                minute: minute as u32,
                second: second as u32,
                nanosecond: nanosecond as u64,
            }
        },
    );

    let hm = map(separated_pair(i32, tag(":"), u32), |(hour, minute)| Time {
        hour: hour as u32,
        minute: minute,
        second: 0,
        nanosecond: 0,
    });

    let x = alt((hms, hm))(input);
    x
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

fn datetime<'a>(input: &'a str, calendar: &'a Calendars) -> IResult<&'a str, CFDatetimes> {
    let cf_datetime_factory = CFDateTimeFactory {};
    fn space1_or_t(input: &str) -> IResult<&str, ()> {
        alt((value((), space1), value((), tag("T"))))(input)
    }
    let tz = map(
        separated_pair(
            separated_pair(|x| date(x, calendar), space1_or_t, time),
            space1,
            timezone,
        ),
        |((date, time), tz)| cf_datetime_factory.build(date, time, tz).unwrap(),
    );

    let no_tz = map(
        separated_pair(|x| date(x, calendar), space1_or_t, time),
        |(date, time)| {
            let tz = Tz { hour: 0, minute: 0 };
            cf_datetime_factory.build(date, time, tz).unwrap()
        },
    );

    let date_with_tz = map(
        separated_pair(|x| date(x, calendar), space1, timezone),
        |(date, tz)| {
            let time = Time {
                hour: 0,
                minute: 0,
                second: 0,
                nanosecond: 0,
            };
            cf_datetime_factory.build(date, time, tz).unwrap()
        },
    );

    let date_time_no_space_tz = map(
        separated_pair(
            separated_pair(|x| date(x, calendar), space1_or_t, time),
            peek(one_of("+-Z")),
            timezone,
        ),
        |((date, time), tz)| cf_datetime_factory.build(date, time, tz).unwrap(),
    );

    let only_date = map(
        |x| date(x, calendar),
        |date| {
            let time = Time {
                hour: 0,
                minute: 0,
                second: 0,
                nanosecond: 0,
            };
            let tz = Tz { hour: 0, minute: 0 };
            cf_datetime_factory.build(date, time, tz).unwrap()
        },
    );

    let x = alt((tz, date_time_no_space_tz, no_tz, date_with_tz, only_date))(input);
    x
}

#[derive(Debug)]
pub enum CFDates {
    DateProlepticGregorian(DateProlepticGregorian),
    DateAllLeap(DateAllLeap),
    DateNoLeap(DateNoLeap),
    Date360Day(Date360Day),
    DateJulian(DateJulian),
}

struct CFDateFactory {
    calendar: Calendars,
}

impl CFDateFactory {
    fn build(&self, year: i32, month: u32, day: u32) -> Option<CFDates> {
        match self.calendar {
            Calendars::ProlepticGregorian => Some(CFDates::DateProlepticGregorian(
                DateProlepticGregorian::new(year, month, day),
            )),
            Calendars::Day360 => Some(CFDates::Date360Day(Date360Day::new(year, month, day))),
            Calendars::Day365 | Calendars::NoLeap => {
                Some(CFDates::DateNoLeap(DateNoLeap::new(year, month, day)))
            }
            Calendars::AllLeap | Calendars::Day366 => {
                Some(CFDates::DateAllLeap(DateAllLeap::new(year, month, day)))
            }
            Calendars::Julian => None,
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum CFDatetimes {
    DateTimeProlepticGregorian(DateTimeProlepticGregorian),
    DateTimeAllLeap(DateTimeAllLeap),
    DateTimeNoLeap(DateTimeNoLeap),
    DateTime360Day(DateTime360Day),
    DateTimeJulian(DateTimeJulian),
}
struct CFDateTimeFactory {}

impl CFDateTimeFactory {
    fn build(&self, date: CFDates, time: Time, tz: Tz) -> Option<CFDatetimes> {
        match date {
            CFDates::DateProlepticGregorian(date) => Some(CFDatetimes::DateTimeProlepticGregorian(
                DateTimeProlepticGregorian::new(date, time, tz),
            )),

            CFDates::Date360Day(date) => Some(CFDatetimes::DateTime360Day(DateTime360Day::new(
                date, time, tz,
            ))),
            CFDates::DateNoLeap(date) => Some(CFDatetimes::DateTimeNoLeap(DateTimeNoLeap::new(
                date, time, tz,
            ))),
            CFDates::DateAllLeap(date) => Some(CFDatetimes::DateTimeAllLeap(DateTimeAllLeap::new(
                date, time, tz,
            ))),
            CFDates::DateJulian(date) => None,
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ParsedCFTime {
    pub duration: CFDuration,
    pub from: CFDatetimes,
}

/// Parse a CF compatible string into two components
pub fn cf_parser(input: &str, calendar: Option<Calendars>) -> Result<ParsedCFTime, ParseError> {
    let since = tuple((space1, tag("since"), space1));
    let calendar = match calendar {
        Some(calendar) => calendar,
        None => Calendars::default(),
    };
    let x = all_consuming(separated_pair(
        |x| duration(x, &calendar),
        since,
        |x| datetime(x, &calendar),
    ))(input)
    .map(|(_, o)| ParsedCFTime {
        duration: o.0,
        from: o.1,
    })
    .map_err(|e| ParseError(format!("{}", e)));
    x
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(input: &str) {
        println!("{:?}", cf_parser(input, None).unwrap())
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
        // parse("days since 1600-02-30 00:00:00");
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
