use std::{
    fmt::{Debug, Display},
    mem::size_of,
};

use windows::Win32::{
    Devices::Bluetooth::{
        BluetoothFindDeviceClose, BluetoothFindFirstDevice, BluetoothFindNextDevice,
        BLUETOOTH_ADDRESS, BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS,
    },
    Foundation::{BOOL, HANDLE},
};

use super::{error::Error, Result, Time};

#[non_exhaustive]
#[allow(dead_code)]
#[derive(Debug)]
/// Lists the possible Bluetooth device classes.
/// This list is incomplete - there are hundreds of device classes, listed in
/// [this
/// spec](https://btprodspecificationrefs.blob.core.windows.net/assigned-numbers/Assigned%20Number%20Types/Assigned_Numbers.pdf)
/// and we only care about a small subset of them. Future devices might be added
/// to this list as I see fit.
pub enum DeviceClass {
    Headset,
    Microphone,
    Speaker,
    Headphones,
    Other,
}

#[derive(Debug)]
/// Represents an active Bluetooth device on the system. This data comes from
/// the Windows API
/// ([windows::Win32::Devices::Bluetooth::BLUETOOTH_DEVICE_INFO]).
pub struct Device {
    pub class: DeviceClass,
    pub address: Address,
    pub connected: bool,
    pub remembered: bool,
    pub authenticated: bool,
    pub name: String,
    pub last_seen: Time,
    pub last_connected: Time,
}

#[allow(dead_code)]
/// Represents a Bluetooth address as a vector of bytes. A Bluetooth address is
/// usually a 48-bit value, but Windows API gives it to us as 6 u8s so this is
/// how we are dealing with it for now.
pub struct Address {
    address: Vec<u8>,
}

impl Address {
    /// Presents the Bluetooth address in the common `11:22:33:44:55:66` format.
    pub fn to_string(&self) -> String {
        self.address
            .iter()
            .map(|v| format!("{:02x}", v).to_string())
            .collect::<Vec<String>>()
            .join(":")
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BluetoothAddress")
            .field("address", &self.to_string())
            .finish()
    }
}

impl From<BLUETOOTH_ADDRESS> for Address {
    /// Converts a [windows::Win32::Devices::Bluetooth::BLUETOOTH_ADDRESS] to
    /// an [Address]. We get the address in reverse byte order from the Win32
    /// API, so it is also reversed to the correct order in this step.
    ///
    /// # Safety
    ///
    /// Microsoft tells us this is safe, and when would they ever lie?
    fn from(value: BLUETOOTH_ADDRESS) -> Self {
        let mut addr = unsafe { value.Anonymous.rgBytes };
        addr.reverse();
        Self {
            address: addr.to_vec(),
        }
    }
}

impl Into<Device> for BLUETOOTH_DEVICE_INFO {
    /// Converts a [windows::Win32::Devices::Bluetooth::BLUETOOTH_DEVICE_INFO]
    /// to a [Device].
    fn into(self) -> Device {
        Device {
            class: from_class_identifier(self.ulClassofDevice),
            address: Address::from(self.Address),
            connected: self.fConnected.into(),
            remembered: self.fRemembered.into(),
            authenticated: self.fAuthenticated.into(),
            name: u16_slice_to_string(self.szName.as_slice()),
            last_seen: self.stLastSeen.into(),
            last_connected: self.stLastUsed.into(),
        }
    }
}

/// Collects a list of all Bluetooth devices currently known to the system on
/// all Bluetooth radios available to the system.
///
/// # Safety
///  
/// There are 3 unsafe blocks in this function.
///
/// * To address the safety in the first block, we initialize the device search
/// params and the device info structs manually.
/// * For the second block, we
/// ensure that there is a valid device handle beforehand, and that we get a
/// valid device as a result.
/// * Finally, the `device_handle` in the
/// `BluetoothFindDeviceClose` call should always be valid as, if it was not,
/// we'd have bailed out earlier.
pub fn get_bluetooth_devices() -> Result<Vec<Device>> {
    let params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
        dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
        fReturnAuthenticated: BOOL::from(true),
        fReturnRemembered: BOOL::from(true),
        fReturnUnknown: BOOL::from(true),
        fReturnConnected: BOOL::from(true),
        fIssueInquiry: BOOL::from(true),
        cTimeoutMultiplier: 1,
        hRadio: HANDLE::default(),
    };

    let mut device_info = BLUETOOTH_DEVICE_INFO::default();
    device_info.dwSize = size_of::<BLUETOOTH_DEVICE_INFO>() as u32;

    let device_handle = unsafe { BluetoothFindFirstDevice(&params, &mut device_info) };
    if device_handle == 0 {
        return Err(Error::NoDevicesFound);
    }

    let mut devices: Vec<Device> = Vec::new();
    devices.push(device_info.into());

    while unsafe { BluetoothFindNextDevice(device_handle, &mut device_info) == BOOL::from(true) } {
        devices.push(device_info.into());
    }

    unsafe {
        BluetoothFindDeviceClose(device_handle);
    }

    Ok(devices)
}

/// Helper method to convert the helpfully null-padded UTF-16 string given to us by the Windows API into a proper Rust [String].
///
/// # Arguments
///
/// * `slice` - A slice of 16-bit integers representing a (hopefully) valid, null-padded UTF-16 string.
fn u16_slice_to_string(slice: &[u16]) -> String {
    String::from_utf16_lossy(slice)
        .trim_matches(char::from(0))
        .to_string()
}

/// Helper method to convert the class identifier number into the device class. Currently only works for two classes...
///
/// # Arguments
///
/// * `identifier` - A 32-bit device class identifier as provided by the [spec](https://btprodspecificationrefs.blob.core.windows.net/assigned-numbers/Assigned%20Number%20Types/Assigned_Numbers.pdf)
fn from_class_identifier(identifier: u32) -> DeviceClass {
    // TODO: proper matching
    match identifier {
        2_360_340 => DeviceClass::Speaker,
        2_360_344 => DeviceClass::Headphones,
        _ => DeviceClass::Other,
    }
}
