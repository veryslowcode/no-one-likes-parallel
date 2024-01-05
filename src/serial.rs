/*******************************************************************************/
/********************************************************************************
* DESCRIPTION: Defines the interaction with the serial interface (e.g., reading,
* writing, etc.).
* AUTHOR: jb
* DATE: 12/30/23
********************************************************************************/
/*******************************************************************************/
use anyhow::{anyhow, Result};
use serialport::{DataBits, Parity as SParity, SerialPortBuilder, StopBits};
use std::time::Duration;
use tokio::{
    self,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::{self, JoinHandle},
};

use crate::common::*;
/******************************************************************************/
/*******************************************************************************
* Public Interface | Implementation
*******************************************************************************/
/******************************************************************************/
#[derive(Debug)]
pub enum PortPacket {
    Data(Vec<u8>),
    Error(String),
}

#[derive(Debug)]
pub struct PortListener {
    port: SerialPortBuilder,
    task: Option<JoinHandle<()>>,
    sender: UnboundedSender<PortPacket>,
    receiver: UnboundedReceiver<PortPacket>,
}

impl PortListener {
    fn new(port: SerialPortBuilder) -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        let tx_handle = tx.clone();
        let port_handle = port.clone();
        let task = Some(PortListener::start(tx_handle, port_handle));

        Ok(PortListener {
            receiver: rx,
            sender: tx,
            port,
            task,
        })
    }

    fn start(
        tx: UnboundedSender<PortPacket>,
        port_builder: SerialPortBuilder,
    ) -> task::JoinHandle<()> {
        let task = tokio::spawn(async move {
            let mut port = match port_builder.open() {
                Ok(p) => p,
                Err(_) => {
                    tx.send(PortPacket::Error(String::from(" Port open failed ")))
                        .expect("Error notify failed");
                    return;
                }
            };
            let mut input_buffer: Vec<u8> = Vec::new();
            loop {
                match port.read(input_buffer.as_mut_slice()) {
                    Ok(_) => {
                        tx.send(PortPacket::Data(input_buffer.clone()))
                            .expect("Input buffer error");
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(_) => {
                        tx.send(PortPacket::Error(String::from(
                            "An unexpected error occured during read",
                        )))
                        .expect("Error notify failed");
                    }
                }
            }
        });
        return task;
    }

    pub async fn listen(&mut self) -> Result<PortPacket> {
        self.receiver
            .recv()
            .await
            .ok_or(anyhow!("Receive packet failed"))
    }
}

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
