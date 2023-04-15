//! Enumerates common errors for this module.

#[derive(Debug)]
/// Enumerates common errors for this module.
pub enum Error {
    /// Returned if there are no Bluetooth devices connected to the system.
    NoDevicesFound,
    /// Returned if there are no Bluetooth radios available to the system.
    NoRadiosFound,
}
