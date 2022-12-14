use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tz {
    pub hour: i8,
    pub minute: u8,
}

impl Default for Tz {
    fn default() -> Self {
        Self { hour: 0, minute: 0 }
    }
}

impl fmt::Display for Tz {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "+{:02}:{:02}", self.hour, self.minute)
    }
}
