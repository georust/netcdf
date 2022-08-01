use crate::calendars::Calendars;
use crate::constants;
use crate::durations::CFDuration;
use crate::time::Time;
use crate::traits::IsLeap;
use crate::{impl_date_display, impl_dt_display, impl_getter};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Tz {
    pub hour: i8,
    pub minute: u8,
}

impl fmt::Display for Tz {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let time_str = format!("+{:02}:{:02}", self.hour, self.minute);
        write!(f, "{}", time_str)
    }
}
