#[derive(Debug, Clone)]
pub struct DisplayStr(String);

impl DisplayStr {
    pub fn new(s: impl ToString) -> Self {
        Self(s.to_string())
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

use std::fmt::{Display, Formatter, Result};
impl Display for DisplayStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}
