#![allow(unused)]
use crate::constants;
use num_integer::div_mod_floor;
use std::fmt;
#[derive(Debug, Clone, Copy)]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub nanosecond: u64,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{:06}",
            self.hour, self.minute, self.second, self.nanosecond
        )
    }
}

impl Default for Time {
    fn default() -> Self {
        Self {
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
        }
    }
}

impl Time {
    pub fn new(hour: u32, minute: u32, second: u32, nanosecond: u64) -> Self {
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
    pub fn nanosecond(&self) -> u64 {
        self.nanosecond
    }
    pub fn num_hours(&self) -> u32 {
        self.hour
    }
    pub fn num_minutes(&self) -> u32 {
        self.hour * 24 + self.minute
    }
    pub fn num_seconds(&self) -> u32 {
        self.num_minutes() * 60 + self.second
    }
    pub fn num_nanoseconds(&self) -> u32 {
        (self.num_seconds() as f64 * 1e6) as u32
    }
    pub fn from_timestamp(seconds: i32) -> Self {
        // Positive modulo (i % n + n) % n
        let _mod_sec = constants::SECS_PER_DAY as i32;
        let seconds = (seconds % _mod_sec + _mod_sec) % _mod_sec;
        let (mins, sec) = div_mod_floor(seconds, 60);
        let (hour, min) = div_mod_floor(mins, 60);
        Self::new(hour as u32, min as u32, sec as u32, 0)
    }
}
