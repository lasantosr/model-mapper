use std::convert::Infallible;

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, Utc};

use super::ExtraInto;

impl ExtraInto<NaiveDateTime> for DateTime<Utc> {
    fn into_extra(self) -> NaiveDateTime {
        self.naive_utc()
    }
}

impl ExtraInto<NaiveDateTime> for DateTime<FixedOffset> {
    fn into_extra(self) -> NaiveDateTime {
        self.naive_utc()
    }
}

impl ExtraInto<DateTime<FixedOffset>> for NaiveDateTime {
    fn into_extra(self) -> DateTime<FixedOffset> {
        self.and_utc().fixed_offset()
    }
}

/// Mapper between `NaiveDateTime` and `DateTime<Utc>`
pub fn chrono_naive_to_utc(from: NaiveDateTime) -> DateTime<Utc> {
    from.and_utc()
}

/// Mapper between `NaiveDateTime` and `DateTime<Utc>`
pub fn try_chrono_naive_to_utc(from: NaiveDateTime) -> Result<DateTime<Utc>, Infallible> {
    Ok(from.and_utc())
}

/// Mapper between `DateTime<Utc>` and `NaiveDateTime`
pub fn chrono_utc_to_naive(from: DateTime<Utc>) -> NaiveDateTime {
    from.naive_utc()
}

/// Mapper between `DateTime<Utc>` and `NaiveDateTime`
pub fn try_chrono_utc_to_naive(from: DateTime<Utc>) -> Result<NaiveDateTime, Infallible> {
    Ok(from.naive_utc())
}

/// Mapper between `NaiveDateTime` and `DateTime<FixedOffset>`
pub fn chrono_naive_to_fixed_offset(from: NaiveDateTime) -> DateTime<FixedOffset> {
    from.and_utc().fixed_offset()
}

/// Mapper between `NaiveDateTime` and `DateTime<FixedOffset>`
pub fn try_chrono_naive_to_fixed_offset(from: NaiveDateTime) -> Result<DateTime<FixedOffset>, Infallible> {
    Ok(from.and_utc().fixed_offset())
}

/// Mapper between `DateTime<FixedOffset>` and `NaiveDateTime`
pub fn chrono_fixed_offset_to_naive(from: DateTime<FixedOffset>) -> NaiveDateTime {
    from.naive_utc()
}

/// Mapper between `DateTime<FixedOffset>` and `NaiveDateTime`
pub fn try_chrono_fixed_offset_to_naive(from: DateTime<FixedOffset>) -> Result<NaiveDateTime, Infallible> {
    Ok(from.naive_utc())
}

/// Mapper between `Duration` and seconds in `i64`
pub fn chrono_duration_to_seconds(from: Duration) -> i64 {
    from.num_seconds()
}

/// Mapper between `Duration` and seconds in `i64`
pub fn try_chrono_duration_to_seconds(from: Duration) -> Result<i64, Infallible> {
    Ok(from.num_seconds())
}

/// Mapper between seconds in `i64` and `Duration`
pub fn seconds_to_chrono_duration(from: i64) -> Duration {
    Duration::seconds(from)
}

/// Mapper between seconds in `i64` and `Duration`
pub fn try_seconds_to_chrono_duration(from: i64) -> Result<Duration, Infallible> {
    Ok(Duration::seconds(from))
}

/// Mapper between `Duration` and milliseconds in `i64`
pub fn chrono_duration_to_millis(from: Duration) -> i64 {
    from.num_milliseconds()
}

/// Mapper between `Duration` and milliseconds in `i64`
pub fn try_chrono_duration_to_millis(from: Duration) -> Result<i64, Infallible> {
    Ok(from.num_milliseconds())
}

/// Mapper between milliseconds in `i64` and `Duration`
pub fn millis_to_chrono_duration(from: i64) -> Duration {
    Duration::milliseconds(from)
}

/// Mapper between milliseconds in `i64` and `Duration`
pub fn try_millis_to_chrono_duration(from: i64) -> Result<Duration, Infallible> {
    Ok(Duration::milliseconds(from))
}
