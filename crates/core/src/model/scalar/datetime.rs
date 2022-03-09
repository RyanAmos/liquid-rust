use std::fmt;
use std::ops;

mod strftime;

use super::Date;

/// Liquid's native date + time type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
#[repr(transparent)]
pub struct DateTime {
    #[serde(with = "friendly_date_time")]
    inner: DateTimeImpl,
}

type DateTimeImpl = time::OffsetDateTime;

impl DateTime {
    /// Create a `DateTime` from the current moment.
    pub fn now() -> Self {
        Self {
            inner: DateTimeImpl::now_utc(),
        }
    }

    /// Convert a `str` to `Self`
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(other: &str) -> Option<Self> {
        parse_date_time(other).map(|d| Self { inner: d })
    }

    /// Replace date with `other`.
    pub fn with_date(self, other: Date) -> Self {
        Self {
            inner: self.inner.replace_date(other.inner),
        }
    }

    /// Changes the associated time zone. This does not change the actual DateTime (but will change the string representation).
    pub fn with_offset(self, offset: time::UtcOffset) -> Self {
        Self {
            inner: self.inner.to_offset(offset),
        }
    }

    /// Retrieves a date component.
    pub fn date(self) -> Date {
        Date {
            inner: self.inner.date(),
        }
    }

    /// Formats the combined date and time with the specified format string.
    ///
    /// See the [chrono::format::strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
    /// module on the supported escape sequences.
    #[inline]
    pub fn format(&self, fmt: &str) -> Result<String, strftime::DateFormatError> {
        strftime::strftime(self.inner, fmt)
    }
}

impl Default for DateTime {
    fn default() -> Self {
        Self {
            inner: DateTimeImpl::UNIX_EPOCH,
        }
    }
}

const DATE_TIME_FORMAT: &[time::format_description::FormatItem<'static>] = time::macros::format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"
);

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.inner
                .format(DATE_TIME_FORMAT)
                .map_err(|_e| fmt::Error)?
        )
    }
}

impl ops::Deref for DateTime {
    type Target = DateTimeImpl;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for DateTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

mod friendly_date_time {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(date: &DateTimeImpl, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date
            .format(DATE_TIME_FORMAT)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTimeImpl, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: std::borrow::Cow<'_, str> = Deserialize::deserialize(deserializer)?;
        DateTimeImpl::parse(&s, DATE_TIME_FORMAT).map_err(serde::de::Error::custom)
    }
}

fn parse_date_time(s: &str) -> Option<DateTimeImpl> {
    const USER_FORMATS: &[&[time::format_description::FormatItem<'_>]] = &[
        time::macros::format_description!("[day] [month repr:long] [year] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"),
        time::macros::format_description!("[day] [month repr:short] [year] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"),
        DATE_TIME_FORMAT,
    ];

    match s {
        "now" => Some(DateTimeImpl::now_utc()),
        _ => USER_FORMATS
            .iter()
            .filter_map(|f| DateTimeImpl::parse(s, f).ok())
            .next(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_date_time_empty_is_bad() {
        let input = "";
        let actual = parse_date_time(input);
        assert!(actual.is_none());
    }

    #[test]
    fn parse_date_time_bad() {
        let input = "aaaaa";
        let actual = parse_date_time(input);
        assert!(actual.is_none());
    }

    #[test]
    fn parse_date_time_now() {
        let input = "now";
        let actual = parse_date_time(input);
        assert!(actual.is_some());
    }

    #[test]
    fn parse_date_time_serialized_format() {
        let input = "2016-02-16 10:00:00 +0100";
        let actual = parse_date_time(input);
        assert!(actual.is_some());
    }

    #[test]
    fn parse_date_time_to_string() {
        let date = DateTime::now();
        let input = date.to_string();
        let actual = parse_date_time(&input);
        assert!(actual.is_some());
    }
}
