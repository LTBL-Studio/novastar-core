//! This crate contains the logic to discover and connect to Novastar [Controller]s through the network

use std::{
    io::{Error, Read, Write},
    net::{SocketAddr, TcpStream, UdpSocket},
    time::Duration,
};

use num_enum::TryFromPrimitive;

use crate::{
    controller::Controller,
    novastarpacket::{NovastarPacket, build_tx_sender},
    types::{FeatureAddress, OpCode, SenderCardType},
};

/// Returns an Iterator over the Novastar [Controller]s available on the network
pub fn discover() -> Result<impl Iterator<Item = Controller>, Error> {
    let discover_socket: UdpSocket = UdpSocket::bind("0.0.0.0:3800")?;
    discover_socket.set_broadcast(true)?;
    discover_socket.set_read_timeout(Some(Duration::from_secs(1)))?;
    let mut controllers = Vec::new();

    let tx_discover_buffer: [u8; 8] = [0x72, 0x71, 0x50, 0x72, 0x6f, 0x4d, 0x49, 0x3a]; //rqProMi:

    discover_socket.send_to(&tx_discover_buffer, "255.255.255.255:3800")?;
    let mut rx_discover_buffer = [0; 16];
    while let Ok((_rx_bytes_count, mut rx_host)) =
        discover_socket.recv_from(&mut rx_discover_buffer)
    {
        rx_host.set_port(5200);
        if let Ok(controller) = try_tcp_connect(rx_host)
            && let Some(controller) = controller
        {
            controllers.push(controller)
        }
    }

    Ok(controllers.into_iter())
}

/// Tries to connect to a Novastar [Controller] on the provided [SocketAddr]
/// # Returns
/// - an [Option] containintg the [Controller] if it exists on the provided socket address
/// - an [Error] if the TCP connexion fails
pub fn try_tcp_connect(socket: SocketAddr) -> Result<Option<Controller>, Error> {
    let mut stream = TcpStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    let tx_buff = build_tx_sender(
        OpCode::Read,
        0x00,
        FeatureAddress::ControllerModelIdAddr,
        &[0, 0],
    );

    stream.write_all(&tx_buff)?;
    stream.flush()?;
    let rx_buff: &mut [u8; 22] = &mut [0; 22];
    stream.read_exact(rx_buff)?;
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
            connexion: crate::ConnexionType::Tcp(socket, stream),
        }
    }))
}
