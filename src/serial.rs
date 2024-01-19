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
use std::{io::ErrorKind, sync::Arc, thread, time::Duration};

use crate::common::*;
/******************************************************************************/
/*******************************************************************************
* Public Interface | Implementation
*******************************************************************************/
/******************************************************************************/
pub fn close_connection(flag: &SerialFlag) -> bool {
    let mut success = false;
    let mut f_lock = flag.try_lock();
    if let Ok(ref mut f_mutex) = f_lock {
        **f_mutex = false;
        success = true;
        drop(f_lock);
    }
    return success;
}

pub fn get_available_devices() -> Result<Vec<String>> {
    let mut devices = Vec::new();
    let ports = serialport::available_ports()?;
    for port in ports {
        devices.push(port.port_name);
    }
    return Ok(devices);
}

pub fn get_error(error: &SerialError) -> Option<String> {
    let mut e_lock = error.try_lock();
    if let Ok(ref mut e_mutex) = e_lock {
        if (**e_mutex).is_some() {
            let msg = (**e_mutex).clone().unwrap();
            (**e_mutex) = None;
            return Some(msg);
        }
        drop(e_lock);
    }
    return None;
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

pub fn open_connection(
    flag: &SerialFlag,
    serial_params: &SerialParams,
    port_params: PortParameters,
) -> bool {
    let mut success = false;
    let mut p_lock = serial_params.try_lock();
    let mut f_lock = flag.try_lock();
    if let Ok(ref mut p_mutex) = p_lock {
        if let Ok(ref mut f_mutex) = f_lock {
            **p_mutex = port_params.clone();
            **f_mutex = true;
            success = true;
        }         
        drop(f_lock);
        drop(p_lock);
    }     
    return success;
}

pub fn read_write_port(
    port: SerialPortBuilder,
    rx: &SerialBuffer,
    tx: &SerialBuffer,
    flag: &SerialFlag,
    error: &SerialError,
) -> thread::JoinHandle<()> {
    let rx_handle = Arc::clone(&rx);
    let tx_handle = Arc::clone(&tx);
    let f_handle = Arc::clone(&flag);
    let e_handle = Arc::clone(&error);
    thread::spawn(move || {
        let mut connection;
        let mut f = true;
        let f_lock = f_handle.try_lock();
        if let Ok(ref f_mutex) = f_lock {
            f = (**f_mutex).clone();
            drop(f_lock);
        }
        match port.open() {
            Ok(c) => connection = c,
            Err(_) => {
                let mut e_lock = e_handle.try_lock();
                if let Ok(ref mut e_mutex) = e_lock {
                    **e_mutex = Some(String::from(" Failed to open port "));
                    drop(e_lock);
                }
                return;
            }
        };

        while f == true {
            let mut tx_lock = tx_handle.try_lock();
            if let Ok(ref mut tx_mutex) = tx_lock {
                if (**tx_mutex).len() > 0 {
                    match connection.write(tx_mutex) {
                        Ok(_) => {
                            (**tx_mutex).clear();
                        }
                        Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
                        Err(_) => {
                            let mut e_lock = e_handle.try_lock();
                            if let Ok(ref mut e_mutex) = e_lock {
                                **e_mutex = Some(String::from(" Write failed "));
                                drop(e_lock);
                            }
                        }
                    };
                    (**tx_mutex).clear();
                }
                drop(tx_lock);
            }

            let mut rx_lock = rx_handle.try_lock();
            if let Ok(ref mut rx_mutex) = rx_lock {
                let mut buffer = vec![0; 1];
                match connection.read(buffer.as_mut_slice()) {
                    Ok(_) => {
                        (**rx_mutex).append(&mut buffer);
                    }
                    Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
                    Err(_) => {
                        let mut e_lock = e_handle.try_lock();
                        if let Ok(ref mut e_mutex) = e_lock {
                            **e_mutex = Some(String::from(" Read failed "));
                        }
                    }
                }
                drop(rx_lock);
            }

            let f_lock = f_handle.try_lock();
            if let Ok(ref f_mutex) = f_lock {
                f = (**f_mutex).clone();
                drop(f_lock);
            }

            thread::sleep(Duration::from_millis(10));
        }

        drop(connection);
    })
}
