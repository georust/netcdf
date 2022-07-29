pub const SECS_PER_DAY: i64 = 86400;

// DAYS CALENDARS
pub const DAYS_PER_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub const CUM_DAYS_PER_MONTH: [u32; 13] =
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
pub const DAYS_PER_MONTH_LEAP: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
pub const CUM_DAYS_PER_MONTH_LEAP: [u32; 13] =
    [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];
pub const DAYS_PER_MONTH_360: [u32; 12] = [30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30];
pub const CUM_DAYS_PER_MONTH_360: [u32; 13] =
    [0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330, 360];

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
