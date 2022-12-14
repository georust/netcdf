#![allow(unused)]
use std::fmt;

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

const DEFAULT_CAL: Calendars = Calendars::ProlepticGregorian;

impl Default for Calendars {
    fn default() -> Calendars {
        DEFAULT_CAL
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
