use crate::calendars::Calendars;
use crate::constants;
use crate::durations::CFDuration;
use crate::time::Time;
use crate::traits::{DateLike, DateTimeLike, IsLeap};
use crate::tz::Tz;
use crate::{impl_date_display, impl_dt_display, impl_getter};
use num_integer::div_mod_floor;
use std::{
    fmt,
    ops::{Add, Sub},
};
#[derive(Debug, Copy, Clone, Default)]
pub struct DateProlepticGregorian {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl DateProlepticGregorian {
    const DAYS_PER_MONTH: [u32; 12] = constants::DAYS_PER_MONTH;
    const CUM_DAYS_PER_MONTH: [u32; 13] = constants::CUM_DAYS_PER_MONTH;
    const DAYS_PER_MONTH_LEAP: [u32; 12] = constants::DAYS_PER_MONTH_LEAP;
    const CUM_DAYS_PER_MONTH_LEAP: [u32; 13] = constants::CUM_DAYS_PER_MONTH_LEAP;
    const CALENDAR: Calendars = Calendars::ProlepticGregorian;

    pub fn new(year: i32, month: u32, day: u32) -> DateProlepticGregorian {
        if (month > 12) | (month < 1) {
            panic!("Month should be between 1 and 12. Found {month}")
        }
        let max_day = match DateProlepticGregorian::is_leap(year) {
            true => DateProlepticGregorian::DAYS_PER_MONTH_LEAP[(month - 1) as usize],
            false => DateProlepticGregorian::DAYS_PER_MONTH[(month - 1) as usize],
        };
        if day > max_day {
            panic!(
                "Day can not exceed {max_day} for {} of the year {year} and {}",
                constants::MONTHS[(month - 1) as usize],
                DateProlepticGregorian::CALENDAR
            )
        }
        Self {
            year: year,
            month: month,
            day: day,
        }
    }
}
impl DateLike for DateProlepticGregorian {
    fn num_days_from_ce(&self) -> i32 {
        let dy = self.year - constants::UNIX_DEFAULT_YEAR;
        let mut num_days: u32 = (dy.abs() * 365) as u32;
        for i in 0..dy.abs() {
            if DateProlepticGregorian::is_leap(constants::UNIX_DEFAULT_YEAR + i) {
                num_days += 1;
            }
        }
        if DateProlepticGregorian::is_leap(constants::UNIX_DEFAULT_YEAR) {
            num_days += constants::CUM_DAYS_PER_MONTH_LEAP[(self.month - 1) as usize]
        } else {
            num_days += constants::CUM_DAYS_PER_MONTH[(self.month - 1) as usize]
        }
        // First begin to 1
        num_days += self.day - 1;
        (num_days as i32) * dy.signum()
    }
    fn num_hours_from_ce(&self) -> i32 {
        self.num_days_from_ce() * 24
    }
    fn num_minutes_from_ce(&self) -> i32 {
        self.num_hours_from_ce() * 60
    }
    fn num_seconds_from_ce(&self) -> i32 {
        self.num_minutes_from_ce() * 60
    }
    fn num_nanoseconds_from_ce(&self) -> i64 {
        ((self.num_seconds_from_ce() as f64) * 1e6) as i64
    }

    fn from_timestamp(seconds: i32) -> DateProlepticGregorian {
        let (nb_days, seconds) = div_mod_floor(seconds, constants::SECS_PER_DAY as i32);

        let nb_non_leap_years = nb_days / 365;

        let mut year = constants::UNIX_DEFAULT_YEAR + nb_non_leap_years;

        let mut year_start = constants::UNIX_DEFAULT_YEAR;
        let mut year_end = constants::UNIX_DEFAULT_YEAR + nb_non_leap_years;

        let mut nb_of_leap_year = 0;
        if year_end < year_start {
            (year_start, year_end) = (year_end, year_start);
        }

        for year in year_start..year_end {
            if DateProlepticGregorian::is_leap(year) {
                nb_of_leap_year += 1;
            }
        }
        let mut remaining_days = nb_days % 365;
        if remaining_days.is_negative() {
            year -= 1;
            // remaining_days += 365 + nb_of_leap_year
            remaining_days = remaining_days + 365 + nb_of_leap_year + 1
        } else {
            remaining_days = remaining_days - nb_of_leap_year + 1
        }
        // if 365 then 0;
        // Do not include 00:00:00 as previous day
        if seconds == 0 && nb_days.is_negative() && DateProlepticGregorian::is_leap(year) {
            remaining_days += 1;
        }

        let mut month: u32 = 1;
        if DateProlepticGregorian::is_leap(year) {
            for v in DateProlepticGregorian::DAYS_PER_MONTH_LEAP.iter() {
                if remaining_days - (*v as i32) >= 0 && ((month + 1) <= 12) {
                    remaining_days -= *v as i32;
                    month += 1
                } else {
                    break;
                }
            }
        } else {
            for v in DateProlepticGregorian::DAYS_PER_MONTH.iter() {
                if remaining_days - (*v as i32) > 0 {
                    remaining_days -= *v as i32;
                    month += 1;
                } else {
                    break;
                }
            }
        }
        DateProlepticGregorian::new(year, month, remaining_days as u32)
    }
}
impl IsLeap for DateProlepticGregorian {
    fn is_leap(year: i32) -> bool {
        let mut f_year = year;
        if year < 0 {
            f_year = year + 1;
        }
        (f_year % 400 == 0) | ((f_year % 4 == 0) && (f_year % 100 != 0))
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateTimeProlepticGregorian {
    pub date: DateProlepticGregorian,
    pub time: Time,
    pub tz: Tz,
}

impl DateTimeProlepticGregorian {
    pub fn new(date: DateProlepticGregorian, time: Time, tz: Tz) -> DateTimeProlepticGregorian {
        DateTimeProlepticGregorian {
            date: date,
            time: time,
            tz: tz,
        }
    }
}
impl DateTimeLike for DateTimeProlepticGregorian {
    fn from_hms(hour: u32, minute: u32, second: u32) -> Self {
        Self {
            date: DateProlepticGregorian::new(
                constants::UNIX_DEFAULT_YEAR,
                constants::UNIX_DEFAULT_MONTH,
                constants::UNIX_DEFAULT_DAY,
            ),
            time: Time::new(hour, minute, second, 0),
            tz: Tz { hour: 0, minute: 0 },
        }
    }
    fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self {
            date: DateProlepticGregorian::new(year, month, day),
            time: Time::new(0, 0, 0, 0),
            tz: Tz { hour: 0, minute: 0 },
        }
    }
    fn from_timestamp(seconds: i32) -> Self {
        Self {
            date: DateProlepticGregorian::from_timestamp(seconds),
            time: Time::from_timestamp(seconds),
            tz: Tz { hour: 0, minute: 0 },
        }
    }
    fn num_hours_from_ce(&self) -> i32 {
        self.date.num_hours_from_ce() + (self.time.num_hours() as i32)
    }
    fn num_minutes_from_ce(&self) -> i32 {
        self.date.num_minutes_from_ce() + (self.time.num_minutes() as i32)
    }
    fn num_seconds_from_ce(&self) -> i32 {
        self.date.num_seconds_from_ce() + (self.time.num_seconds() as i32)
    }
    fn num_nanoseconds_from_ce(&self) -> i64 {
        self.date.num_nanoseconds_from_ce() + (self.time.num_nanoseconds() as i64)
    }
}

/// As nanos field is private this is a solution to emulate it
fn _get_real_nano_field(duration: CFDuration) -> i64 {
    let chrono_time = chrono::Duration::seconds(duration.num_seconds());
    let ns = (duration
        - CFDuration {
            duration: chrono_time,
            calendar: duration.calendar,
        })
    .num_nanoseconds()
    .unwrap();
    ns
}

impl Add<CFDuration> for DateTimeProlepticGregorian {
    type Output = Self;
    fn add(self, other: CFDuration) -> Self {
        let ns = _get_real_nano_field(other);
        let mut dt = DateTimeProlepticGregorian::from_timestamp(
            self.num_seconds_from_ce() + other.num_seconds() as i32,
        );
        dt.time.nanosecond = ns as u64;
        dt
    }
}

impl Sub<CFDuration> for DateTimeProlepticGregorian {
    type Output = Self;
    fn sub(self, other: CFDuration) -> Self {
        let ns = _get_real_nano_field(other);
        let mut timestamp = self.num_seconds_from_ce() - other.num_seconds() as i32;
        if ns > 0 {
            timestamp -= 1
        }
        let mut dt = DateTimeProlepticGregorian::from_timestamp(timestamp);
        if ns > 0 {
            dt.time.nanosecond = (constants::MAX_NS - other.num_nanoseconds().unwrap()) as u64;
        }
        dt
    }
}

impl_getter!(DateProlepticGregorian);
impl_date_display!(DateProlepticGregorian);
impl_dt_display!(DateTimeProlepticGregorian);

#[cfg(test)]
mod test {
    use super::*;
    use crate::durations::CFDuration;
    #[test]
    fn test_add_duration_to_datetime() {
        let dt = DateTimeProlepticGregorian::from_timestamp(0);
        let dur = CFDuration::days(1, Calendars::ProlepticGregorian);
        let new_dt = dt + dur;
        assert_eq!(new_dt.date.year, 1970);
        assert_eq!(new_dt.date.month, 01);
        assert_eq!(new_dt.date.day, 02);
        assert_eq!(new_dt.time.hour, 00);
        assert_eq!(new_dt.time.minute, 00);
        assert_eq!(new_dt.time.second, 00);
        let dt = DateTimeProlepticGregorian::from_timestamp(0);
        let dur = CFDuration::milliseconds(1, Calendars::ProlepticGregorian);
        let new_dt = dt + dur;
        assert_eq!(new_dt.date.year, 1970);
        assert_eq!(new_dt.date.month, 01);
        assert_eq!(new_dt.date.day, 01);
        assert_eq!(new_dt.time.hour, 00);
        assert_eq!(new_dt.time.minute, 00);
        assert_eq!(new_dt.time.second, 00);
        assert_eq!(new_dt.time.nanosecond, 1000000);
    }
    #[test]
    fn test_sub_duration_to_datetime() {
        let dt = DateTimeProlepticGregorian::from_timestamp(0);
        let dur = CFDuration::days(1, Calendars::ProlepticGregorian);
        let new_dt = dt - dur;
        println!("{new_dt}");
        assert_eq!(new_dt.date.year, 1969);
        assert_eq!(new_dt.date.month, 12);
        assert_eq!(new_dt.date.day, 31);
        assert_eq!(new_dt.time.hour, 00);
        assert_eq!(new_dt.time.minute, 00);
        assert_eq!(new_dt.time.second, 00);
        let dt = DateTimeProlepticGregorian::from_timestamp(0);
        let dur = CFDuration::milliseconds(1, Calendars::ProlepticGregorian);
        let new_dt = dt - dur;
        assert_eq!(new_dt.date.year, 1969);
        assert_eq!(new_dt.date.month, 12);
        assert_eq!(new_dt.date.day, 31);
        assert_eq!(new_dt.time.hour, 23);
        assert_eq!(new_dt.time.minute, 59);
        assert_eq!(new_dt.time.second, 59);
        assert_eq!(new_dt.time.nanosecond, 999000000);
    }

    #[test]
    fn test_from_timestamp() {
        let dt = DateTimeProlepticGregorian::from_timestamp(0);
        println!("{dt}");
        assert_eq!(dt.date.year, 1970);
        assert_eq!(dt.date.month, 01);
        assert_eq!(dt.date.day, 01);
        assert_eq!(dt.time.hour, 00);
        assert_eq!(dt.time.minute, 00);
        assert_eq!(dt.time.second, 00);
        // Bug found for this value
        let dt = DateTimeProlepticGregorian::from_timestamp(-86400);
        println!("{dt}");
        assert_eq!(dt.date.year, 1969);
        assert_eq!(dt.date.month, 12);
        assert_eq!(dt.date.day, 31);
        assert_eq!(dt.time.hour, 00);
        assert_eq!(dt.time.minute, 00);
        assert_eq!(dt.time.second, 00);
        let dt = DateTimeProlepticGregorian::from_timestamp(-1);
        assert_eq!(dt.date.year, 1969);
        assert_eq!(dt.date.month, 12);
        assert_eq!(dt.date.day, 31);
        assert_eq!(dt.time.hour, 23);
        assert_eq!(dt.time.minute, 59);
        assert_eq!(dt.time.second, 59);
        let dt = DateTimeProlepticGregorian::from_timestamp(1000000);
        println!("{dt}");
        assert_eq!(dt.date.year, 1970);
        assert_eq!(dt.date.month, 01);
        assert_eq!(dt.date.day, 12);
        assert_eq!(dt.time.hour, 13);
        assert_eq!(dt.time.minute, 46);
        assert_eq!(dt.time.second, 40);
        let dt = DateTimeProlepticGregorian::from_timestamp(1658876523);
        println!("{dt}");
        assert_eq!(dt.date.year, 2022);
        assert_eq!(dt.date.month, 07);
        assert_eq!(dt.date.day, 26);
        assert_eq!(dt.time.hour, 23);
        assert_eq!(dt.time.minute, 02);
        assert_eq!(dt.time.second, 03);
        let dt = DateTimeProlepticGregorian::from_timestamp(-1658876523);
        println!("{dt}");
        assert_eq!(dt.date.year, 1917);
        assert_eq!(dt.date.month, 06);
        assert_eq!(dt.date.day, 08);
        assert_eq!(dt.time.hour, 00);
        assert_eq!(dt.time.minute, 57);
        assert_eq!(dt.time.second, 57);
    }
}
