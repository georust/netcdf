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
                write!(f, "{} {}{}", self.date, self.time, self.tz)
            }
        }
    };
}
pub(crate) use impl_date_display;
pub(crate) use impl_dt_display;
pub(crate) use impl_getter;
