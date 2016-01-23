use std::error::Error;
use std::fmt;
use std::convert::From;
use std::io;
use nbtrs;

// TODO figure out our set of error
// This current definition is just a generic container for an error message
#[derive(Debug)]
pub struct OverviewerError {
    msg: String,
}

impl Error for OverviewerError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for OverviewerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "OverviewerError: {}", self.msg)
    }
}

// macro to quickly produce a From impl for error types
// this little hack will go away once we have better error handling
macro_rules! ErrImpl {
    ( $t:ty, $e:expr ) => (
        impl From<$t> for OverviewerError {
            fn from(e: $t) -> OverviewerError {
                OverviewerError{msg: format!("{}: {}", $e, e.description())}
            }
        }
    );
    ( $t:ty, $e:expr, debug ) => (
        impl From<$t> for OverviewerError {
            fn from(e: $t) -> OverviewerError {
                OverviewerError{msg: format!("{}: {:?}", $e, e)}
            }
        }
    );
    ( $t:ty ) => (
        impl From<$t> for OverviewerError {
            fn from(e: $t) -> OverviewerError {
                OverviewerError{msg: From::from(e)}
            }
        }
    );
}

ErrImpl!(&'static str);
ErrImpl!(String);
ErrImpl!(io::Error, "IOError");
ErrImpl!(nbtrs::Error, "NBTError", debug);
