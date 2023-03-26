use chrono::prelude::*;
use std::{
    fmt::{Debug, Display},
    mem::size_of,
};
use windows::Win32::{
    Devices::Bluetooth::{
        BluetoothFindFirstDevice, BluetoothFindNextDevice, BLUETOOTH_ADDRESS,
        BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS,
    },
    Foundation::{BOOL, HANDLE, SYSTEMTIME},
};

type Result<T> = std::result::Result<T, BluetoothError>;

#[derive(Debug)]
struct SystemTime(NaiveDateTime);
impl std::ops::Deref for SystemTime {
    type Target = NaiveDateTime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
enum BluetoothError {
    NoDevicesFound,
}

fn main() -> Result<()> {
    let devices = get_bluetooth_devices()?;
    println!("{:?}", devices);
    Ok(())
}

#[non_exhaustive]
#[allow(dead_code)]
#[derive(Debug)]
enum BluetoothDeviceClass {
    Headset,
    Microphone,
    Speaker,
    Headphones,
    Other,
}

#[derive(Debug)]
#[allow(dead_code)]
struct BluetoothDevice {
    class: BluetoothDeviceClass,
    address: BluetoothAddress,
    connected: bool,
    remembered: bool,
    authenticated: bool,
    name: String,
    last_seen: SystemTime,
    last_connected: SystemTime,
}

#[allow(dead_code)]
struct BluetoothAddress {
    address: Vec<u8>,
}

impl BluetoothAddress {
    fn to_string(&self) -> String {
        self.address
            .iter()
            .map(|v| format!("{:02x}", v).to_string())
            .collect::<Vec<String>>()
            .join(":")
    }
}

impl Display for BluetoothAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Debug for BluetoothAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BluetoothAddress")
            .field("address", &self.to_string())
            .finish()
    }
}

impl From<BLUETOOTH_ADDRESS> for BluetoothAddress {
    fn from(value: BLUETOOTH_ADDRESS) -> Self {
        let mut addr = unsafe { value.Anonymous.rgBytes }; // Microsoft tells us this is safe, and when have they ever lied?
        addr.reverse();
        Self {
            address: addr.to_vec(),
        }
    }
}

impl Into<SystemTime> for SYSTEMTIME {
    fn into(self) -> SystemTime {
        SystemTime(
            NaiveDate::from_ymd_opt(self.wYear.into(), self.wMonth.into(), self.wDay.into())
                .unwrap()
                .and_hms_opt(self.wHour.into(), self.wMinute.into(), self.wSecond.into())
                .unwrap()
                .into(),
        )
    }
}

impl Into<BluetoothDevice> for BLUETOOTH_DEVICE_INFO {
    fn into(self) -> BluetoothDevice {
        BluetoothDevice {
            class: from_class_identifier(self.ulClassofDevice),
            address: BluetoothAddress::from(self.Address),
            connected: self.fConnected.into(),
            remembered: self.fRemembered.into(),
            authenticated: self.fAuthenticated.into(),
            name: u16_slice_to_string(self.szName.as_slice()),
            last_seen: self.stLastSeen.into(),
            last_connected: self.stLastUsed.into(),
        }
    }
}

fn get_bluetooth_devices() -> Result<Vec<BluetoothDevice>> {
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
        return Err(BluetoothError::NoDevicesFound);
    }

    let mut devices: Vec<BluetoothDevice> = Vec::new();
    devices.push(device_info.into());

    while unsafe { BluetoothFindNextDevice(device_handle, &mut device_info) == BOOL::from(true) } {
        devices.push(device_info.into());
    }

    Ok(devices)
}

fn u16_slice_to_string(slice: &[u16]) -> String {
    String::from_utf16_lossy(slice)
        .trim_matches(char::from(0))
        .to_string()
}

fn from_class_identifier(identifier: u32) -> BluetoothDeviceClass {
    // https://btprodspecificationrefs.blob.core.windows.net/assigned-numbers/Assigned%20Number%20Types/Assigned_Numbers.pdf
    // TODO: proper matching
    match identifier {
        2_360_340 => BluetoothDeviceClass::Speaker,
        2_360_344 => BluetoothDeviceClass::Headphones,
        _ => BluetoothDeviceClass::Other,
    }
}
