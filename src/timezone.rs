#[cfg(not(feature = "chrono"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Utc;

#[cfg(feature = "chrono")]
pub use chrono::Utc;

#[cfg(feature = "chrono")]
pub use chrono_tz::*;
