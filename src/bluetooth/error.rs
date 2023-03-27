#[derive(Debug)]
/// Enumerates common errors for this crate.
pub enum Error {
    /// Returned if there are no Bluetooth devices connected to the system.
    NoDevicesFound,
}
