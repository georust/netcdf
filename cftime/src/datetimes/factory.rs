#![allow(unused)]
use crate::calendars::Calendars;
use crate::datetimes::all_leap::{DateAllLeap, DateTimeAllLeap};
use crate::datetimes::day360::{Date360Day, DateTime360Day};
use crate::datetimes::julian::{DateJulian, DateTimeJulian};
use crate::datetimes::no_leap::{DateNoLeap, DateTimeNoLeap};
use crate::datetimes::prolecpticgregorian::{DateProlepticGregorian, DateTimeProlepticGregorian};
use crate::time::Time;
use crate::tz::Tz;
#[derive(Debug)]
pub enum CFDates {
    DateProlepticGregorian(DateProlepticGregorian),
    DateAllLeap(DateAllLeap),
    DateNoLeap(DateNoLeap),
    Date360Day(Date360Day),
    DateJulian(DateJulian),
}

#[derive(Debug)]
pub enum CFDatetimes {
    DateTimeProlepticGregorian(DateTimeProlepticGregorian),
    DateTimeAllLeap(DateTimeAllLeap),
    DateTimeNoLeap(DateTimeNoLeap),
    DateTime360Day(DateTime360Day),
    DateTimeJulian(DateTimeJulian),
}

pub struct CFDateFactory {}

impl CFDateFactory {
    pub fn build(year: i32, month: u32, day: u32, calendar: Calendars) -> Option<CFDates> {
        match calendar {
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

pub struct CFDateTimeFactory {}

impl CFDateTimeFactory {
    pub fn build(date: CFDates, time: Time, tz: Tz) -> Option<CFDatetimes> {
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
