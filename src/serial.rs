/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Defines the interaction with the serial interface (e.g., reading,
* writing, etc.).
* AUTHOR: jb
* DATE: 12/30/23
********************************************************************************/
/*******************************************************************************/
use anyhow::Result;
use serialport::{DataBits, Parity as SParity, SerialPortBuilder, StopBits};
use std::time::Duration;

use crate::common::*;
/******************************************************************************/
/*******************************************************************************
* Public Interface | Implementation
*******************************************************************************/
/******************************************************************************/
pub fn get_available_devices() -> Result<Vec<String>> {
    let mut devices = Vec::new();
    let ports = serialport::available_ports()?;
    for port in ports {
        devices.push(port.port_name);
    }
    return Ok(devices);
}

pub fn get_port(parameters: PortParameters) -> Result<SerialPortBuilder> {
    let timeout = Duration::from_secs(10);
    let data_bits = match parameters.data_bits.unwrap() {
        5 => DataBits::Five,
        6 => DataBits::Six,
        7 => DataBits::Seven,
        8 => DataBits::Eight,
        _ => unreachable!(),
    };
    let stop_bits = match parameters.stop_bits.unwrap() {
        1 => StopBits::One,
        2 => StopBits::Two,
        _ => unreachable!(),
    };
    let parity = match parameters.parity.unwrap() {
        Parity::Even => SParity::Even,
        Parity::Odd => SParity::Odd,
        Parity::None => SParity::None,
    };

    let port = serialport::new(parameters.name.unwrap(), parameters.baud_rate.unwrap())
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .timeout(timeout);

    return Ok(port);
}
