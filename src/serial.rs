use anyhow::Result;
use serialport;

pub fn get_available_devices() -> Result<Vec<String>> {
    let mut devices = Vec::new();
    let ports = serialport::available_ports()?;
    for port in ports {
        devices.push(port.port_name);
    }
    return Ok(devices);
}
