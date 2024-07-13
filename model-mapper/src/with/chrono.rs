use chrono::{DateTime, Duration, FixedOffset, Local, NaiveDateTime, TimeZone, Utc};

use super::TypeMapper;

/// Mapper between different types of chrono date time
pub struct DateTimeMapper;
impl<T: TimeZone> TypeMapper<DateTime<T>, NaiveDateTime> for DateTimeMapper {
    fn map(from: DateTime<T>) -> NaiveDateTime {
        from.naive_utc()
    }
}
impl TypeMapper<DateTime<Utc>, DateTime<FixedOffset>> for DateTimeMapper {
    fn map(from: DateTime<Utc>) -> DateTime<FixedOffset> {
        from.fixed_offset()
    }
}
impl TypeMapper<DateTime<Utc>, DateTime<Local>> for DateTimeMapper {
    fn map(from: DateTime<Utc>) -> DateTime<Local> {
        from.into()
    }
}
impl TypeMapper<DateTime<FixedOffset>, DateTime<Utc>> for DateTimeMapper {
    fn map(from: DateTime<FixedOffset>) -> DateTime<Utc> {
        from.to_utc()
    }
}
impl TypeMapper<DateTime<FixedOffset>, DateTime<Local>> for DateTimeMapper {
    fn map(from: DateTime<FixedOffset>) -> DateTime<Local> {
        from.into()
    }
}
impl TypeMapper<DateTime<Local>, DateTime<Utc>> for DateTimeMapper {
    fn map(from: DateTime<Local>) -> DateTime<Utc> {
        from.to_utc()
    }
}
impl TypeMapper<DateTime<Local>, DateTime<FixedOffset>> for DateTimeMapper {
    fn map(from: DateTime<Local>) -> DateTime<FixedOffset> {
        from.fixed_offset()
    }
}
impl TypeMapper<NaiveDateTime, DateTime<Utc>> for DateTimeMapper {
    fn map(from: NaiveDateTime) -> DateTime<Utc> {
        from.and_utc()
    }
}
impl TypeMapper<NaiveDateTime, DateTime<FixedOffset>> for DateTimeMapper {
    fn map(from: NaiveDateTime) -> DateTime<FixedOffset> {
        from.and_utc().fixed_offset()
    }
}
impl TypeMapper<NaiveDateTime, DateTime<Local>> for DateTimeMapper {
    fn map(from: NaiveDateTime) -> DateTime<Local> {
        from.and_utc().into()
    }
}

/// Mapper between chrono types and seconds
pub struct SecondsMapper;
impl TypeMapper<Duration, i64> for SecondsMapper {
    fn map(from: Duration) -> i64 {
        from.num_seconds()
    }
}
impl TypeMapper<i64, Duration> for SecondsMapper {
    fn map(from: i64) -> Duration {
        Duration::seconds(from)
    }
}
impl<T: TimeZone> TypeMapper<DateTime<T>, i64> for SecondsMapper {
    fn map(from: DateTime<T>) -> i64 {
        from.timestamp_millis() / 1000
    }
}
impl TypeMapper<NaiveDateTime, i64> for SecondsMapper {
    fn map(from: NaiveDateTime) -> i64 {
        from.and_utc().timestamp_millis() / 1000
    }
}

/// Mapper between chrono types and milliseconds
pub struct MillisecondsMapper;
impl TypeMapper<Duration, i64> for MillisecondsMapper {
    fn map(from: Duration) -> i64 {
        from.num_milliseconds()
    }
}
impl TypeMapper<i64, Duration> for MillisecondsMapper {
    fn map(from: i64) -> Duration {
        Duration::milliseconds(from)
    }
}
impl<T: TimeZone> TypeMapper<DateTime<T>, i64> for MillisecondsMapper {
    fn map(from: DateTime<T>) -> i64 {
        from.timestamp_millis()
    }
}
impl TypeMapper<NaiveDateTime, i64> for MillisecondsMapper {
    fn map(from: NaiveDateTime) -> i64 {
        from.and_utc().timestamp_millis()
    }
}
