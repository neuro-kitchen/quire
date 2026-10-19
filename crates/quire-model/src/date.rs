//! [`Date`]: a calendar date known to the year, month or day (`2017`, `2017-06`, `2017-06-12`),
//! written as in BibLaTeX's `date` field and ISO 8601.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{ModelError, Result};

/// A date with year and optionally month and day. Years 1–9999.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Date {
    year: u16,
    month: Option<u8>,
    day: Option<u8>,
}

impl Date {
    /// Fails for a year outside 1–9999, a month outside 1–12, a day not in that month, or a day
    /// without a month.
    pub fn new(year: u16, month: Option<u8>, day: Option<u8>) -> Result<Self> {
        let invalid = |reason| ModelError::InvalidDate { input: format!("{year}-{month:?}-{day:?}"), reason };
        if !(1..=9999).contains(&year) {
            return Err(invalid("year must be 1 to 9999"));
        }
        match (month, day) {
            (Some(m), _) if !(1..=12).contains(&m) => return Err(invalid("month must be 1 to 12")),
            (None, Some(_)) => return Err(invalid("a day needs a month")),
            (Some(m), Some(d)) if d == 0 || d > days_in_month(year, m) => {
                return Err(invalid("no such day in that month"));
            }
            _ => {}
        }
        Ok(Self { year, month, day })
    }

    /// A date known only to the year. Panics outside 1–9999 (use [`new`](Self::new) for input).
    pub fn year(year: u16) -> Self {
        Self::new(year, None, None).expect("year in 1..=9999")
    }

    pub fn get_year(self) -> u16 {
        self.year
    }

    pub fn month(self) -> Option<u8> {
        self.month
    }

    pub fn day(self) -> Option<u8> {
        self.day
    }
}

fn days_in_month(year: u16, month: u8) -> u8 {
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}", self.year)?;
        if let Some(m) = self.month {
            write!(f, "-{m:02}")?;
        }
        if let Some(d) = self.day {
            write!(f, "-{d:02}")?;
        }
        Ok(())
    }
}

impl FromStr for Date {
    type Err = ModelError;

    /// Parses `YYYY`, `YYYY-MM` or `YYYY-MM-DD`.
    fn from_str(s: &str) -> Result<Self> {
        let invalid = |reason| ModelError::InvalidDate { input: s.into(), reason };
        let number = |part: &str| -> Result<u16> {
            let digits = !part.is_empty() && part.len() <= 4 && part.bytes().all(|b| b.is_ascii_digit());
            digits.then(|| part.parse().ok()).flatten().ok_or_else(|| invalid("expected YYYY, YYYY-MM or YYYY-MM-DD"))
        };
        let mut parts = s.trim().split('-');
        let year = number(parts.next().unwrap_or_default())?;
        let month = parts.next().map(number).transpose()?.map(|m| m as u8);
        let day = parts.next().map(number).transpose()?.map(|d| d as u8);
        if parts.next().is_some() {
            return Err(invalid("expected YYYY, YYYY-MM or YYYY-MM-DD"));
        }
        Self::new(year, month, day).map_err(|e| match e {
            ModelError::InvalidDate { reason, .. } => invalid(reason),
            other => other,
        })
    }
}

impl TryFrom<String> for Date {
    type Error = ModelError;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<Date> for String {
    fn from(date: Date) -> Self {
        date.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display() {
        for text in ["2017", "2017-06", "2017-06-12", "0800"] {
            assert_eq!(text.parse::<Date>().unwrap().to_string(), text);
        }
    }

    #[test]
    fn rejects_impossible_dates() {
        for text in ["", "17x", "2017-13", "2017-02-30", "2017-00", "0000", "2017-06-12-01", "2017--01"] {
            assert!(text.parse::<Date>().is_err(), "{text}");
        }
        assert!(Date::new(2017, None, Some(3)).is_err());
    }

    #[test]
    fn leap_years() {
        assert!("2024-02-29".parse::<Date>().is_ok());
        assert!("2023-02-29".parse::<Date>().is_err());
        assert!("1900-02-29".parse::<Date>().is_err());
        assert!("2000-02-29".parse::<Date>().is_ok());
    }

    #[test]
    fn orders_by_year_then_month_then_day() {
        let parse = |t: &str| t.parse::<Date>().unwrap();
        assert!(parse("2017") < parse("2017-01"));
        assert!(parse("2017-12-31") < parse("2018"));
    }
}
