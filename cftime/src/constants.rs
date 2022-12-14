#![allow(unused)]
pub const SECS_PER_DAY: i64 = 86400;

const fn cumsum_cal(input: &[u32; 12]) -> [u32; 13] {
    let mut out = [0; 13];
    let mut i = 1;
    while i < 13 {
        out[i] = out[i - 1] + input[i - 1];
        i += 1;
    }
    out
}

// DAYS CALENDARS
pub const DAYS_PER_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub const CUM_DAYS_PER_MONTH: [u32; 13] = cumsum_cal(&DAYS_PER_MONTH);
pub const DAYS_PER_MONTH_LEAP: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub const CUM_DAYS_PER_MONTH_LEAP: [u32; 13] = cumsum_cal(&DAYS_PER_MONTH_LEAP);
pub const DAYS_PER_MONTH_360: [u32; 12] = [30; 12];
pub const CUM_DAYS_PER_MONTH_360: [u32; 13] = cumsum_cal(&DAYS_PER_MONTH_360);

// UNIX TIMESTAMP
pub const UNIX_DEFAULT_YEAR: i32 = 1970;
pub const UNIX_DEFAULT_MONTH: u32 = 01;
pub const UNIX_DEFAULT_DAY: u32 = 01;

pub const MONTHS: [&str; 12] = [
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

pub const MAX_NS: i64 = 1_000_000_000;
