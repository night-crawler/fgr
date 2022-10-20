use chrono::Duration;

use crate::mk_filter_enum;

mk_filter_enum!(TimeUnit, TIME_UNIT_ALIASES, [
    Second: "s", "secs",
    Minute: "m", "min", "mins", "minute",
    Hour: "h", "hour",
    Day: "d", "day"
]);

impl TimeUnit {
    pub fn to_duration(&self, value: i64) -> Duration {
        match self {
            TimeUnit::Second => Duration::seconds(value),
            TimeUnit::Minute => Duration::minutes(value),
            TimeUnit::Hour => Duration::hours(value),
            TimeUnit::Day => Duration::days(value),
        }
    }
}
