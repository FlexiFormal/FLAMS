use std::fmt::Display;
use std::num::{NonZeroU64};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timestamp(NonZeroU64);
impl Timestamp {
    pub fn now() -> Self {
        let t = chrono::Utc::now().timestamp_millis() as u64;
        Self(NonZeroU64::new(t).unwrap())
    }
    pub fn since_now(&self) -> Delta {
        let t = self.0.get() as i64;
        let n = chrono::Utc::now().timestamp_millis();
        Delta(NonZeroU64::new((n - t) as u64).unwrap())
    }
    pub fn since(&self,o:Timestamp) -> Delta {
        let o = o.0.get() as i64;
        let s = self.0.get() as i64;
        Delta(NonZeroU64::new((s-o) as u64).unwrap_or(NonZeroU64::new(1).unwrap()))
    }
}
impl FromStr for Timestamp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = chrono::DateTime::<chrono::Utc>::from_str(s).map_err(|_| ())?;
        let m = c.timestamp_millis();
        let t = m as u64;
        Ok(Self(NonZeroU64::new(t).ok_or_else(|| ())?))
    }
}
impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts = self.0.get() as i64;
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ts).unwrap()
            .with_timezone(&chrono::Local);
        timestamp.format("%Y-%m-%d %H:%M:%S").fmt(f)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Delta(NonZeroU64);
impl Display for Delta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = chrono::Duration::milliseconds(self.0.get() as i64);
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;
        let seconds = duration.num_seconds() % 60;
        let millis = duration.num_milliseconds() % 1000;

        let mut is_empty = true;

        if days > 0 {
            is_empty = false;
            f.write_str(&format!("{:02}d ", days))?;
        }
        if hours > 0 {
            is_empty = false;
            f.write_str(&format!("{:02}h ", hours))?;
        }
        if minutes > 0 {
            is_empty = false;
            f.write_str(&format!("{:02}m ", minutes))?;
        }
        if seconds > 0 {
            is_empty = false;
            f.write_str(&format!("{:02}s ", seconds))?;
        }
        if millis > 0 {
            is_empty = false;
            f.write_str(&format!("{:02}ms", millis))?;
        }
        if is_empty { f.write_str("0ms")?; }

        Ok(())
    }
}