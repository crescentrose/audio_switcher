mod bluetooth;

use crate::bluetooth::device::get_bluetooth_devices;
use crate::bluetooth::radio::get_bluetooth_radio;
use crate::bluetooth::Result;

fn main() -> Result<()> {
    let radio = get_bluetooth_radio()?;
    let devices = get_bluetooth_devices()?;
    for device in devices {
        device.get_device_services(&radio);
    }

    Ok(())
}
