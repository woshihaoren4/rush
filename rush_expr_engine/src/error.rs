use std::error::Error;
use std::fmt;

#[derive(Default, Debug)]
pub struct NotFoundFieldError(pub String);

impl fmt::Display for NotFoundFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not found field[{}]", self.0.as_str())
    }
}

impl Error for NotFoundFieldError {}
