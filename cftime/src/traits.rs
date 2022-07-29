use crate::calendars::Calendars;
pub trait IsLeap {
    fn is_leap(year: i32) -> bool;
}

pub trait CFTimeEncoder {
    fn encode(value: i32, unit: &str, calendar: Calendars);
}
pub trait CFTimeDecoder {
    fn decode(unit: &str, calendar: Calendars);
}
