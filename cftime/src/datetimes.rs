use std::{fmt, thread::panicking};

const SEC_PER_DAYS: f64 = 8.64e4;
const SEC_PER_HOURS: f64 = 3.6e3;
const SEC_PER_MINUTES: f64 = 60.0;
const SEC_PER_SECONDS: f64 = 1.0;
const SEC_PER_MILLISECONDS: f64 = 1e-3;
const SEC_PER_MICROSECONDS: f64 = 1e-6;
const SEC_TO_US: f64 = 1e6;

const DAYS_PER_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const CUM_DAYS_PER_MONTH: [u32; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
const DAYS_PER_MONTH_LEAP: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const CUM_DAYS_PER_MONTH_LEAP: [u32; 13] =
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];
const DAYS_PER_MONTH_360: [u32; 12] = [30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30];
const CUM_DAYS_PER_MONTH_360: [u32; 13] =
    [0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330, 360];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

// UNIX TIMESTAMP
const DEFAULT_YEAR: i32 = 1970;
const DEFAULT_MONTH: u32 = 01;
const DEFAULT_DAY: u32 = 01;
const SECONDS_IN_DAYS: u32 = 86_400;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Calendars {
    Gregorian, // alias of Standard
    Standard,
    ProlepticGregorian,
    NoLeap, // 365 days
    Day365,
    AllLeap, // 366 days
    Day366,
    Julian,
    Day360,
}

impl Default for Calendars {
    fn default() -> Calendars {
        Calendars::ProlepticGregorian
    }
}

impl fmt::Display for Calendars {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let name = match *self {
            Calendars::Gregorian => "Gregorian",
            Calendars::Standard => "Standard",
            Calendars::ProlepticGregorian => "Proleptic Gregorian",
            Calendars::NoLeap | Calendars::Day365 => "No Leap",
            Calendars::AllLeap | Calendars::Day366 => "All Leap",
            Calendars::Julian => "Julian",
            Calendars::Day360 => "360 Day",
        };
        write!(f, "{name}")
    }
}
/// Base duration between time points
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum DurationNames {
    Years,
    Months,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Duration {
    pub name: DurationNames,
    pub calendar: Calendars,
}

impl Duration {
    fn _year_in_us(&self) -> f64 {
        match self.calendar {
            Calendars::Gregorian => 365.2425 * SEC_PER_DAYS * SEC_TO_US,
            Calendars::ProlepticGregorian | Calendars::Standard => 3.15569259747e7 * SEC_TO_US,
            Calendars::NoLeap | Calendars::Day365 => 365.0 * SEC_PER_DAYS * SEC_TO_US,
            Calendars::AllLeap | Calendars::Day366 => 366.0 * SEC_PER_DAYS * SEC_TO_US,
            Calendars::Julian => 365.25 * SEC_PER_DAYS * SEC_TO_US,
            Calendars::Day360 => 360.0 * SEC_PER_DAYS * SEC_TO_US,
        }
    }

    fn _month_in_us(&self) -> f64 {
        match self.calendar {
            Calendars::Gregorian => self._year_in_us() / 12.0,
            Calendars::ProlepticGregorian | Calendars::Standard => self._year_in_us() / 12.0,
            Calendars::NoLeap | Calendars::Day365 => self._year_in_us() / 12.0,
            Calendars::AllLeap | Calendars::Day366 => self._year_in_us() / 12.0,
            Calendars::Julian => self._year_in_us() / 12.0,
            Calendars::Day360 => self._year_in_us() / 12.0,
        }
    }
    pub fn microseconds(&self) -> i64 {
        let us = match self.name {
            DurationNames::Days => SEC_PER_DAYS * SEC_TO_US,
            DurationNames::Hours => SEC_PER_HOURS * SEC_TO_US,
            DurationNames::Minutes => SEC_PER_MINUTES * SEC_TO_US,
            DurationNames::Seconds => SEC_PER_SECONDS * SEC_TO_US,
            DurationNames::Milliseconds => SEC_PER_MILLISECONDS * SEC_TO_US,
            DurationNames::Microseconds => SEC_PER_MICROSECONDS * SEC_TO_US,
            DurationNames::Years => self._year_in_us(),
            DurationNames::Months => self._month_in_us(),
        };
        us.floor() as i64
    }

    pub fn seconds(&self) -> i64 {
        ((self.microseconds() as f64) * (1e-6 as f64)) as i64
    }

    pub fn nanoseconds(&self) -> i64 {
        ((self.microseconds() as f64) * (1e3 as f64)) as i64
    }
    pub fn milliseconds(&self) -> i64 {
        ((self.microseconds() as f64) * (1e-3 as f64)) as i64
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub nanosecond: u32,
}

impl fmt::Display for Time {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let time_str = format!(
            "{:02}:{:02}:{:02}.{:06}",
            self.hour, self.minute, self.second, self.nanosecond
        );
        write!(f, "{}", time_str)
    }
}
impl Time {
    pub fn new(hour: u32, minute: u32, second: u32, nanosecond: u32) -> Self {
        if hour >= 24 {
            panic!("Hours should be between 0 and 23. Found {hour}")
        }
        if minute >= 60 {
            panic!("Minutes should be between 0 and 59. Found {minute}")
        }
        if second >= 60 {
            panic!("Seconds should be between 0 and 59. Found {second}")
        }
        if nanosecond >= 1_000_000_000 {
            panic!("Nano-seconds should be between 0 and 1 000 000 000. Found {nanosecond}")
        }
        Self {
            hour,
            minute,
            second,
            nanosecond,
        }
    }
    pub fn hour(&self) -> u32 {
        self.hour
    }
    pub fn minute(&self) -> u32 {
        self.minute
    }
    pub fn second(&self) -> u32 {
        self.second
    }
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Tz {
    pub hour: i8,
    pub minute: u8,
}

trait IsLeap {
    fn is_leap(year: i32) -> bool;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateProlepticGregorian {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl DateProlepticGregorian {
    const DAYS_PER_MONTH: [u32; 12] = DAYS_PER_MONTH;
    const CUM_DAYS_PER_MONTH: [u32; 13] = CUM_DAYS_PER_MONTH;
    const DAYS_PER_MONTH_LEAP: [u32; 12] = DAYS_PER_MONTH_LEAP;
    const CUM_DAYS_PER_MONTH_LEAP: [u32; 13] = CUM_DAYS_PER_MONTH_LEAP;
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
                MONTHS[(month - 1) as usize],
                DateProlepticGregorian::CALENDAR
            )
        }
        Self {
            year: year,
            month: month,
            day: day,
        }
    }
    pub fn num_days_from_ce(&self) -> i32 {
        let dy = self.year - DEFAULT_YEAR;
        let mut num_days: u32 = (dy.abs() * 365) as u32;
        for i in 0..dy.abs() {
            if DateProlepticGregorian::is_leap(DEFAULT_YEAR + i) {
                num_days += 1;
            }
        }
        if DateProlepticGregorian::is_leap(DEFAULT_YEAR) {
            num_days += CUM_DAYS_PER_MONTH_LEAP[(self.month - 1) as usize]
        } else {
            num_days += CUM_DAYS_PER_MONTH[(self.month - 1) as usize]
        }
        // First begin to 1
        num_days += self.day - 1;
        (num_days as i32) * dy.signum()
    }

    pub fn num_seconds_from_ce(&self) -> i32 {
        self.num_days_from_ce() * (SECONDS_IN_DAYS as i32)
    }

    pub fn from_timestamp(seconds: i32) -> DateProlepticGregorian {
        let mut nb_days = seconds / (SECONDS_IN_DAYS as i32);
        let seconds_remaining = seconds % (SECONDS_IN_DAYS as i32);

        let nb_non_leap_years = nb_days / 365;

        let mut f_year = DEFAULT_YEAR + nb_non_leap_years;
        let mut nb_of_leap_year = 0;

        let mut year_start = DEFAULT_YEAR;
        let mut year_end = DEFAULT_YEAR + nb_non_leap_years;

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
            f_year -= 1;
            remaining_days += 365 - nb_of_leap_year
        } else {
            remaining_days += nb_of_leap_year + 1
        }

        // Do not include 00:00:00 as previous day
        if seconds_remaining == 0 && nb_days.is_negative() {
            remaining_days += 1;
        }

        let mut f_month: u32 = 1;
        if DateProlepticGregorian::is_leap(f_year) {
            for v in DateProlepticGregorian::DAYS_PER_MONTH_LEAP.iter() {
                if remaining_days - (*v as i32) > 0 {
                    remaining_days -= (*v as i32);
                    f_month += 1
                } else {
                    break;
                }
            }
        } else {
            for v in DateProlepticGregorian::DAYS_PER_MONTH.iter() {
                if remaining_days - (*v as i32) > 0 {
                    remaining_days -= (*v as i32);
                    f_month += 1
                } else {
                    break;
                }
            }
        }
        DateProlepticGregorian::new(f_year, f_month, remaining_days as u32)
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
    pub fn from_hms(hour: u32, minute: u32, second: u32) -> Self {
        Self {
            date: DateProlepticGregorian::new(DEFAULT_YEAR, DEFAULT_MONTH, DEFAULT_DAY),
            time: Time::new(hour, minute, second, 0),
            tz: Tz { hour: 0, minute: 0 },
        }
    }
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self {
            date: DateProlepticGregorian::new(year, month, day),
            time: Time::new(0, 0, 0, 0),
            tz: Tz { hour: 0, minute: 0 },
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateJulian {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl DateJulian {
    const DAYS_PER_MONTH: [u32; 12] = DAYS_PER_MONTH;
    const CUM_DAYS_PER_MONTH: [u32; 13] = CUM_DAYS_PER_MONTH;
    const DAYS_PER_MONTH_LEAP: [u32; 12] = DAYS_PER_MONTH_LEAP;
    const CUM_DAYS_PER_MONTH_LEAP: [u32; 13] = CUM_DAYS_PER_MONTH_LEAP;
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

#[derive(Debug, Copy, Clone, Default)]
pub struct DateTimeJulian {
    pub date: DateJulian,
    pub time: Time,
    pub tz: Tz,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateAllLeap {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}
impl DateAllLeap {
    const DAYS_PER_MONTH_LEAP: [u32; 12] = DAYS_PER_MONTH_LEAP;
    const CUM_DAYS_PER_MONTH_LEAP: [u32; 13] = CUM_DAYS_PER_MONTH_LEAP;
    const CALENDAR: Calendars = Calendars::AllLeap;
}
#[allow(unused_variables)]
impl IsLeap for DateAllLeap {
    fn is_leap(year: i32) -> bool {
        true
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateTimeAllLeap {
    pub date: DateAllLeap,
    pub time: Time,
    pub tz: Tz,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateNoLeap {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub time: Time,
    pub tz: Tz,
}
impl DateNoLeap {
    const DAYS_PER_MONTH: [u32; 12] = DAYS_PER_MONTH;
    const CUM_DAYS_PER_MONTH: [u32; 13] = CUM_DAYS_PER_MONTH;
    const CALENDAR: Calendars = Calendars::NoLeap;
}

#[allow(unused_variables)]
impl IsLeap for DateNoLeap {
    fn is_leap(year: i32) -> bool {
        false
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateTimeNoLeap {
    pub date: DateNoLeap,
    pub time: Time,
    pub tz: Tz,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Date360Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}
impl Date360Day {
    const DAYS_PER_MONTH_360: [u32; 12] = DAYS_PER_MONTH_360;
    const CUM_DAYS_PER_MONTH_360: [u32; 13] = CUM_DAYS_PER_MONTH_360;
    const CALENDAR: Calendars = Calendars::Day360;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct DateTime360Day {
    pub date: Date360Day,
    pub time: Time,
    pub tz: Tz,
}

macro_rules! impl_getter {
    ($date:ident) => {
        impl $date {
            pub fn year(&self) -> i32 {
                self.year
            }
            pub fn month(&self) -> u32 {
                self.month
            }
            pub fn day(&self) -> u32 {
                self.day
            }
        }
    };
}
macro_rules! impl_date_display {
    ($date:ident) => {
        impl fmt::Display for $date {
            // This trait requires `fmt` with this exact signature.
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                // Write strictly the first element into the supplied output
                // stream: `f`. Returns `fmt::Result` which indicates whether the
                // operation succeeded or failed. Note that `write!` uses syntax which
                // is very similar to `println!`.
                write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
            }
        }
    };
}

macro_rules! impl_dt_display {
    ($datetime:ident) => {
        impl fmt::Display for $datetime {
            // This trait requires `fmt` with this exact signature.
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                // Write strictly the first element into the supplied output
                // stream: `f`. Returns `fmt::Result` which indicates whether the
                // operation succeeded or failed. Note that `write!` uses syntax which
                // is very similar to `println!`.
                let date_str = format!("{}", self.date);
                let time_str = format!("{}", self.time);
                write!(f, "{} {}", date_str, time_str)
            }
        }
    };
}

impl_getter!(DateJulian);
impl_getter!(DateProlepticGregorian);
impl_getter!(DateAllLeap);
impl_getter!(DateNoLeap);
impl_getter!(Date360Day);

impl_date_display!(DateJulian);
impl_date_display!(DateProlepticGregorian);
impl_date_display!(DateAllLeap);
impl_date_display!(DateNoLeap);
impl_date_display!(Date360Day);

impl_dt_display!(DateTimeJulian);
impl_dt_display!(DateTimeProlepticGregorian);
impl_dt_display!(DateTimeAllLeap);
impl_dt_display!(DateTimeNoLeap);
impl_dt_display!(DateTime360Day);

#[derive(Debug)]
pub enum CFDatetimes {
    DateTimeProlepticGregorian(DateTimeProlepticGregorian),
    DateTimeAllLeap(DateTimeAllLeap),
    DateTimeNoLeap(DateTimeNoLeap),
    DateTime360Day(DateTime360Day),
    DateTimeJulian(DateTimeJulian),
}

#[derive(Debug)]
pub struct ParsedCFTime {
    pub duration: Duration,
    pub from: CFDatetimes,
}
trait CFTimeEncoder {
    fn encode(value: i32, unit: &str, calendar: Calendars);
}
trait CFTimeDecoder {
    fn decode(unit: &str, calendar: Calendars);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn duration_month_every_calendar() {
        let duration = Duration {
            name: DurationNames::Months,
            calendar: Calendars::ProlepticGregorian,
        };
        assert_eq!(duration.seconds(), 2629743);
        let duration = Duration {
            name: DurationNames::Months,
            calendar: Calendars::Gregorian,
        };
        assert_eq!(duration.seconds(), 2629746);
        let duration = Duration {
            name: DurationNames::Months,
            calendar: Calendars::AllLeap,
        };
        assert_eq!(duration.seconds(), 2635200);
        let duration = Duration {
            name: DurationNames::Months,
            calendar: Calendars::NoLeap,
        };
        assert_eq!(duration.seconds(), 2628000);
        let duration = Duration {
            name: DurationNames::Months,
            calendar: Calendars::Day360,
        };
        assert_eq!(duration.seconds(), 2592000);
    }
}
