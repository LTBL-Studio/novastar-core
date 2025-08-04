//! This crate contains the logic to discover and connect to Novastar [Controller]s through serial ports

use num_enum::TryFromPrimitive;
use std::{io::Error, time::Duration};

use crate::{
    controller::Controller,
    novastarpacket::{NovastarPacket, build_tx_sender},
    types::{FeatureAddress, OpCode, SenderCardType},
};

/// Returns an Iterator over the Novastar [Controller]s available on the serial ports of this machine
pub fn discover() -> Result<impl Iterator<Item = Controller>, Error> {
    serialport::available_ports()
        .map(|ports| {
            ports.into_iter().filter_map(|port_info| {
                let port_name = port_info.port_name.as_str();
                try_com_connect(port_name, 1048576)
                    .or_else(|_| try_com_connect(port_name, 115200))
                    .ok()
                    .flatten()
            })
        })
        .map_err(|err| err.into())
}

/// Tries to connect to a Novastar [Controller] on the provided port name at the provided baud rate
/// # Returns
/// - an [Option] containing the [Controller] if it exists on the provided port name
/// - an [Error] if the serial connexion fails
pub fn try_com_connect(port_name: &str, baud_rate: u32) -> Result<Option<Controller>, Error> {
    let mut port = serialport::new(port_name, baud_rate).open()?;
    port.set_timeout(Duration::from_secs(1))?;
    let tx_buff = build_tx_sender(
        OpCode::Read,
        0x00,
        FeatureAddress::ControllerModelIdAddr,
        &[0, 0],
    );
    port.write_all(&tx_buff)?;
    port.flush()?;
    let rx_buff: &mut [u8; 22] = &mut [0; 22];
    port.read_exact(rx_buff)?;

    Ok(NovastarPacket::decode(rx_buff).ok().map(|packet| {
        let dev_id: u16 = u16::from_le_bytes([packet.data[0], packet.data[1]]);
        let dev_model =
            SenderCardType::try_from_primitive(dev_id).unwrap_or(SenderCardType::Unknown);
        if dev_model == SenderCardType::Unknown {
            println!("Controller returned model ID {dev_id}");
            println!("{:01$x} ", dev_id, 2);
        }
        Controller {
            card_type: dev_model,
            connexion: crate::ConnexionType::Serial(port_name.to_string(), port),
        }
    }))
}
