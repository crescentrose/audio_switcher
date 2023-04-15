//! Connect to the Bluetooth radio available on the system.

use std::fmt::Debug;
use std::mem::size_of;

use windows::Win32::Devices::Bluetooth::{
    BluetoothFindFirstRadio, BluetoothFindRadioClose, BluetoothGetRadioInfo,
    BLUETOOTH_FIND_RADIO_PARAMS, BLUETOOTH_RADIO_INFO,
};
use windows::Win32::Foundation::{CloseHandle, HANDLE};

use super::error::Error;
use super::{util, Result};

/// Represents a Bluetooth radio connected to the system.
#[derive(Debug)]
pub struct Radio {
    pub name: String,
    pub handle: HANDLE,
}

impl Drop for Radio {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}

/// Gets the first Bluetooth radio plugged into the system.
///
/// According to [Microsoft's own
/// documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/bluetooth/bluetooth-faq),
/// "The Bluetooth stack in Windows supports only one Bluetooth radio".
/// This is the use case that this tool is targeting anyway. But in
/// case someone really insists on adding more than one radio, they should
/// amend this function.
///
/// # Safety
///
/// This method calls the Win32 C API and, therefore, contains several
/// `unsafe` blocks. We need to take care that the
/// [windows::Win32::Devices::Bluetooth::BLUETOOTH_FIND_RADIO_PARAMS]
/// struct is initialized and valid, and that the handle returned is
/// properly closed at the end.
pub fn get_bluetooth_radio() -> Result<Radio> {
    let find_params = BLUETOOTH_FIND_RADIO_PARAMS {
        dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
    };

    let mut radio_info = BLUETOOTH_RADIO_INFO::default();
    radio_info.dwSize = size_of::<BLUETOOTH_RADIO_INFO>() as u32;

    let mut radio = HANDLE::default();
    let find_handle = unsafe { BluetoothFindFirstRadio(&find_params, &mut radio) };
    if find_handle == 0 {
        return Err(Error::NoRadiosFound);
    }

    unsafe { BluetoothGetRadioInfo(radio, &mut radio_info) };
    unsafe { BluetoothFindRadioClose(find_handle) };

    Ok(Radio {
        handle: radio,
        name: util::u16_slice_to_string(radio_info.szName.as_slice()),
    })
}
