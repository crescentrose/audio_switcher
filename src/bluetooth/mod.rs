//! List available Bluetooth devices and connect to them. It supports one
//! Bluetooth radio per system.

pub mod device;
pub mod error;
pub mod radio;
pub mod util;

use std::ops::Deref;

use chrono::{NaiveDate, NaiveDateTime};
use windows::Win32::Foundation::SYSTEMTIME;

/// A classic Rust [Result](std::result::Result) enriched with the Bluetooth module's [Error](error::Error) type.
pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug)]
/// Wraps the [chrono::naive::NaiveDateTime] struct to represent local system time.
pub struct Time(pub NaiveDateTime);

impl Deref for Time {
    type Target = NaiveDateTime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<Time> for SYSTEMTIME {
    fn into(self) -> Time {
        Time(
            NaiveDate::from_ymd_opt(self.wYear.into(), self.wMonth.into(), self.wDay.into())
                .unwrap()
                .and_hms_opt(self.wHour.into(), self.wMinute.into(), self.wSecond.into())
                .unwrap()
                .into(),
        )
    }
}
