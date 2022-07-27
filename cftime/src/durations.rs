use crate::calendars::Calendars;
use chrono;
use std::{
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

const SECS_PER_DAY: i64 = 86400;

/// Base duration between time points. Higly inspired by https://docs.rs/time/0.1.37/src/time/duration.rs.html
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct CFDuration {
    duration: chrono::Duration,
    calendar: Calendars,
}

impl CFDuration {
    /// Makes a new `Duration` with given number of years.
    /// Depends on the calendars definitions found in https://github.com/nco/nco/blob/master/data/udunits.dat
    /// Panics when the duration is out of bounds.
    fn years(years: i64, calendar: Calendars) -> CFDuration {
        let secs_per_year = match calendar {
            Calendars::Gregorian => 365.2425 * SECS_PER_DAY as f64,
            Calendars::ProlepticGregorian | Calendars::Standard => 3.15569259747e7,
            Calendars::NoLeap | Calendars::Day365 => 365.0 * SECS_PER_DAY as f64,
            Calendars::AllLeap | Calendars::Day366 => 366.0 * SECS_PER_DAY as f64,
            Calendars::Julian => 365.25 * SECS_PER_DAY as f64,
            Calendars::Day360 => 360.0 * SECS_PER_DAY as f64,
        };
        let secs = (secs_per_year as i64)
            .checked_mul(years)
            .expect("Duration::months out of bounds") as i64;
        CFDuration::seconds(secs, calendar)
    }

    /// Makes a new `Duration` with given number of months.
    /// Depends on the calendars definitions found in https://github.com/nco/nco/blob/master/data/udunits.dat
    /// Panics when the duration is out of bounds.
    fn months(months: i64, calendar: Calendars) -> CFDuration {
        let secs_per_month = match calendar {
            Calendars::Gregorian => CFDuration::years(1, calendar).num_seconds() as f64 / 12.0,
            Calendars::ProlepticGregorian | Calendars::Standard => {
                CFDuration::years(1, calendar).num_seconds() as f64 / 12.0
            }
            Calendars::NoLeap | Calendars::Day365 => {
                CFDuration::years(1, calendar).num_seconds() as f64 / 12.0
            }
            Calendars::AllLeap | Calendars::Day366 => {
                CFDuration::years(1, calendar).num_seconds() as f64 / 12.0
            }
            Calendars::Julian => CFDuration::years(1, calendar).num_seconds() as f64 / 12.0,
            Calendars::Day360 => CFDuration::years(1, calendar).num_seconds() as f64 / 12.0,
        };
        let secs = (secs_per_month as i64)
            .checked_mul(months)
            .expect("Duration::months out of bounds") as i64;
        CFDuration::seconds(secs, calendar)
    }

    /// Makes a new `Duration` with given number of weeks.
    /// Equivalent to `Duration::seconds(weeks * 7 * 24 * 60 * 60)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn weeks(weeks: i64, calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::weeks(weeks),
            calendar: calendar,
        }
    }

    /// Makes a new `Duration` with given number of days.
    /// Equivalent to `Duration::seconds(days * 24 * 60 * 60)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn days(days: i64, calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::days(days),
            calendar: calendar,
        }
    }

    /// Makes a new `Duration` with given number of hours.
    /// Equivalent to `Duration::seconds(hours * 60 * 60)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn hours(hours: i64, calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::hours(hours),
            calendar: calendar,
        }
    }

    /// Makes a new `Duration` with given number of minutes.
    /// Equivalent to `Duration::seconds(minutes * 60)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn minutes(minutes: i64, calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::minutes(minutes),
            calendar: calendar,
        }
    }

    /// Makes a new `Duration` with given number of seconds.
    /// Panics when the duration is more than `i64::MAX` milliseconds
    /// or less than `i64::MIN` milliseconds.
    #[inline]
    pub fn seconds(seconds: i64, calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::seconds(seconds),
            calendar: calendar,
        }
    }

    /// Makes a new `Duration` with given number of milliseconds.
    #[inline]
    pub fn milliseconds(milliseconds: i64, calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::milliseconds(milliseconds),
            calendar: calendar,
        }
    }

    /// Makes a new `Duration` with given number of microseconds.
    #[inline]
    pub fn microseconds(microseconds: i64, calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::microseconds(microseconds),
            calendar: calendar,
        }
    }

    /// Makes a new `Duration` with given number of nanoseconds.
    #[inline]
    pub fn nanoseconds(nanos: i64, calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::nanoseconds(nanos),
            calendar: calendar,
        }
    }
    /// Returns the total number of whole weeks in the duration.
    #[inline]
    pub fn num_weeks(&self) -> i64 {
        self.duration.num_weeks()
    }

    /// Returns the total number of whole days in the duration.
    pub fn num_days(&self) -> i64 {
        self.duration.num_days()
    }

    /// Returns the total number of whole hours in the duration.
    #[inline]
    pub fn num_hours(&self) -> i64 {
        self.duration.num_hours()
    }

    /// Returns the total number of whole minutes in the duration.
    #[inline]
    pub fn num_minutes(&self) -> i64 {
        self.duration.num_minutes()
    }

    /// Returns the total number of whole seconds in the duration.
    pub fn num_seconds(&self) -> i64 {
        self.duration.num_seconds()
    }

    /// Returns the total number of whole milliseconds in the duration,
    pub fn num_milliseconds(&self) -> i64 {
        self.duration.num_milliseconds()
    }

    /// Returns the total number of whole microseconds in the duration,
    /// or `None` on overflow (exceeding 2^63 microseconds in either direction).
    pub fn num_microseconds(&self) -> Option<i64> {
        self.duration.num_microseconds()
    }

    /// Returns the total number of whole nanoseconds in the duration,
    /// or `None` on overflow (exceeding 2^63 nanoseconds in either direction).
    pub fn num_nanoseconds(&self) -> Option<i64> {
        self.duration.num_nanoseconds()
    }

    /// The minimum possible `Duration`: `i64::MIN` milliseconds.
    #[inline]
    pub fn min_value(calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::min_value(),
            calendar: calendar,
        }
    }

    /// The maximum possible `Duration`: `i64::MAX` milliseconds.
    #[inline]
    pub fn max_value(calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::max_value(),
            calendar: calendar,
        }
    }

    /// A duration where the stored seconds and nanoseconds are equal to zero.
    #[inline]
    pub fn zero(calendar: Calendars) -> CFDuration {
        CFDuration {
            duration: chrono::Duration::zero(),
            calendar: calendar,
        }
    }

    /// Returns `true` if the duration equals `Duration::zero()`.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.duration.is_zero()
    }
}

impl Neg for CFDuration {
    type Output = CFDuration;

    #[inline]
    fn neg(self) -> CFDuration {
        CFDuration {
            duration: self.duration.neg(),
            calendar: self.calendar,
        }
    }
}

impl Add for CFDuration {
    type Output = CFDuration;

    fn add(self, rhs: CFDuration) -> CFDuration {
        if self.calendar != rhs.calendar {
            panic!("Duration::add Cannot add duration with two different calendar");
        }
        CFDuration {
            duration: self.duration.add(rhs.duration),
            calendar: self.calendar,
        }
    }
}

impl Sub for CFDuration {
    type Output = CFDuration;

    fn sub(self, rhs: CFDuration) -> CFDuration {
        if self.calendar != rhs.calendar {
            panic!("Duration::sub Cannot substract duration with two different calendar");
        }
        CFDuration {
            duration: self.duration.sub(rhs.duration),
            calendar: self.calendar,
        }
    }
}

impl Mul<i32> for CFDuration {
    type Output = CFDuration;

    fn mul(self, rhs: i32) -> CFDuration {
        CFDuration {
            duration: self.duration.mul(rhs),
            calendar: self.calendar,
        }
    }
}

impl Div<i32> for CFDuration {
    type Output = CFDuration;

    fn div(self, rhs: i32) -> CFDuration {
        CFDuration {
            duration: self.duration.div(rhs),
            calendar: self.calendar,
        }
    }
}

impl fmt::Display for CFDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // technically speaking, negative duration is not valid ISO 8601,
        // but we need to print it anyway.
        write!(f, "{} ({})", self.duration, self.calendar)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn duration_year_every_calendar() {
        let duration = CFDuration::years(1, Calendars::ProlepticGregorian);
        assert_eq!(duration.num_seconds(), 31556925);
        let duration = CFDuration::years(1, Calendars::Gregorian);
        assert_eq!(duration.num_seconds(), 31556952);
        let duration = CFDuration::years(1, Calendars::AllLeap);
        assert_eq!(duration.num_seconds(), 31622400);
        let duration = CFDuration::years(1, Calendars::NoLeap);
        assert_eq!(duration.num_seconds(), 31536000);
        let duration = CFDuration::years(1, Calendars::Day360);
        assert_eq!(duration.num_seconds(), 31104000);
    }
    #[test]
    fn duration_month_every_calendar() {
        let duration = CFDuration::months(1, Calendars::ProlepticGregorian);
        assert_eq!(duration.num_seconds(), 2629743);
        let duration = CFDuration::months(1, Calendars::Gregorian);
        assert_eq!(duration.num_seconds(), 2629746);
        let duration = CFDuration::months(1, Calendars::AllLeap);
        assert_eq!(duration.num_seconds(), 2635200);
        let duration = CFDuration::months(1, Calendars::NoLeap);
        assert_eq!(duration.num_seconds(), 2628000);
        let duration = CFDuration::months(1, Calendars::Day360);
        assert_eq!(duration.num_seconds(), 2592000);
    }
}
