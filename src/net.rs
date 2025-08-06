//! This crate contains the logic to discover and connect to Novastar [Controller]s through the network

use std::{
    io::Error,
    net::UdpSocket,
    time::Duration,
};

use crate::
    controller::Controller
;

/// Returns an Iterator over the Novastar [Controller]s available on the network
pub fn discover() -> Result<DiscoverIter, Error> {
    let discover_socket: UdpSocket = UdpSocket::bind("0.0.0.0:3800")?;
    discover_socket.set_broadcast(true)?;
    discover_socket.set_read_timeout(Some(Duration::from_secs(1)))?;

    let tx_discover_buffer: [u8; 8] = [0x72, 0x71, 0x50, 0x72, 0x6f, 0x4d, 0x49, 0x3a]; //rqProMi:

    discover_socket.send_to(&tx_discover_buffer, "255.255.255.255:3800")?;
    
    Ok(DiscoverIter {
        socket: discover_socket,
        buffer: [0; 16],
        ended: false
    })
}

/// Iterator over discovered controllers on local network
pub struct DiscoverIter {
    socket: UdpSocket,
    buffer: [u8; 16],
    ended: bool
}

impl Iterator for DiscoverIter {
    type Item = Controller;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None
        }

        let Ok((_rx_bytes_count, mut rx_host)) = self.socket.recv_from(&mut self.buffer) else {
            self.ended = true;
            return None;
        };

        rx_host.set_port(5200);
        let Ok(controller) = Controller::try_from_tcp_addr(rx_host) else {
            return self.next()
        };

        Some(controller)
    }
}