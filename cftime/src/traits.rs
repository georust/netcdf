#![allow(unused)]
use std::{
    fmt::Debug,
    ops::{Add, Sub},
};

use crate::{
    calendars::Calendars,
    datetimes::{
        day360::{Date360Day, DateTime360Day},
        factory::CFDatetimes,
    },
    durations::CFDuration,
    parser::cf_parser,
};
pub trait IsLeap {
    fn is_leap(year: i32) -> bool;
}

pub trait CFTimeEncoder {
    fn encode(unit: &str, calendar: Calendars);
}
pub trait CFTimeDecoder {
    fn decode(self, unit: &str, calendar: Option<Calendars>);
}

pub trait DateLike: Debug {
    fn num_days_from_ce(&self) -> i32;
    fn num_hours_from_ce(&self) -> i32;
    fn num_minutes_from_ce(&self) -> i32;
    fn num_seconds_from_ce(&self) -> i32;
    fn num_nanoseconds_from_ce(&self) -> i64;
    fn from_timestamp(seconds: i32) -> Self
    where
        Self: Sized;
}
pub trait DateTimeLike: Debug {
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
