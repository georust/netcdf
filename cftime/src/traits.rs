use crate::{
    calendars::Calendars,
    parser::{cf_parser, ParsedCFTime},
    time::Time,
    tz::Tz,
};
pub trait IsLeap {
    fn is_leap(year: i32) -> bool;
}

pub trait CFTimeEncoder {
    fn encode(unit: &str, calendar: Calendars);
}
pub trait CFTimeDecoder {
    fn decode(&self, unit: &str, calendar: Option<Calendars>);
}

pub trait DateLike {
    fn num_days_from_ce(&self) -> i32;
    fn num_hours_from_ce(&self) -> i32;
    fn num_minutes_from_ce(&self) -> i32;
    fn num_seconds_from_ce(&self) -> i32;
    fn num_nanoseconds_from_ce(&self) -> i64;
    fn from_timestamp(seconds: i32) -> Self
    where
        Self: Sized;
}
pub trait DateTimeLike {
    fn from_hms(hour: u32, minute: u32, second: u32) -> Self
    where
        Self: Sized;
    fn from_ymd(year: i32, month: u32, day: u32) -> Self
    where
        Self: Sized;
    fn from_timestamp(seconds: i32) -> Self
    where
        Self: Sized;
    fn num_hours_from_ce(&self) -> i32;
    fn num_minutes_from_ce(&self) -> i32;
    fn num_seconds_from_ce(&self) -> i32;
    fn num_nanoseconds_from_ce(&self) -> i64;
}
impl CFTimeDecoder for f64 {
    fn decode(&self, unit: &str, calendar: Option<Calendars>) {
        let parsed_cf_time = cf_parser(unit, calendar).unwrap();
        let duration: f64 = parsed_cf_time.duration.num_seconds() as f64 * *self;
        let from = parsed_cf_time.from;
    }
}
