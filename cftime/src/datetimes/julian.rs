#![allow(unused)]
use crate::calendars::Calendars;
use crate::constants;
use crate::durations::CFDuration;
use crate::macros::{impl_date_display, impl_dt_display, impl_getter};
use crate::time::Time;
use crate::traits::IsLeap;
use crate::tz::Tz;
use num_integer::div_mod_floor;
use std::{
    fmt,
    ops::{Add, Sub},
};

#[derive(Debug, Copy, Clone)]
pub struct DateJulian {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl DateJulian {
    const DAYS_PER_MONTH: [u32; 12] = constants::DAYS_PER_MONTH;
    const CUM_DAYS_PER_MONTH: [u32; 13] = constants::CUM_DAYS_PER_MONTH;
    const DAYS_PER_MONTH_LEAP: [u32; 12] = constants::DAYS_PER_MONTH_LEAP;
    const CUM_DAYS_PER_MONTH_LEAP: [u32; 13] = constants::CUM_DAYS_PER_MONTH_LEAP;
    const CALENDAR: Calendars = Calendars::Julian;
}
impl IsLeap for DateJulian {
    fn is_leap(year: i32) -> bool {
        let mut f_year = year;
        if year < 0 {
            f_year = year + 1;
        }
        f_year % 4 == 0
    }
}
impl Default for DateJulian {
    fn default() -> Self {
        Self {
            year: constants::UNIX_DEFAULT_YEAR,
            month: constants::UNIX_DEFAULT_MONTH,
            day: constants::UNIX_DEFAULT_DAY,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateTimeJulian {
    pub date: DateJulian,
    pub time: Time,
    pub tz: Tz,
}
impl DateTimeJulian {
    pub fn new(date: DateJulian, time: Time, tz: Tz) -> DateTimeJulian {
        DateTimeJulian {
            date: date,
            time: time,
            tz: tz,
        }
    }
}

impl_getter!(DateJulian);
impl_date_display!(DateJulian);
impl_dt_display!(DateTimeJulian);
