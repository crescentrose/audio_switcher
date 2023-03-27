mod bluetooth;

use crate::bluetooth::device::get_bluetooth_devices;
use crate::bluetooth::Result;

fn main() -> Result<()> {
    let devices = get_bluetooth_devices()?;
    println!("{:?}", devices);
    Ok(())
}
