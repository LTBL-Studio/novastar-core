#![allow(dead_code)]
use std::{io::Error, net::UdpSocket};
use tokio::time::Duration;

pub fn discover() {
    //tokio::spawn(init_discover());
    // match init_discover() {
    //     Ok(_) => (),
    //     Err(e) => return Err(e),
    // };
}

async fn init_discover() -> Result<(), Error> {
    let discover_socket: UdpSocket = match UdpSocket::bind("0.0.0.0:3800") {
        Ok(s) => s,
        Err(e) => return Err(e),
    };
    match discover_socket.set_broadcast(true) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    match discover_socket.set_read_timeout(Some(Duration::from_secs(30))) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    let tx_discover_buffer: [u8; 8] = [0x72,0x71,0x50,0x72,0x6f,0x4d,0x49,0x3a]; //rqProMi:
    loop {
        match discover_socket.send_to(&tx_discover_buffer, "255.255.255.255:3800") {
            Ok(_) => (),
            Err(e) => return Err(e),
        };
        let mut rx_discover_buffer: Vec<u8> = Vec::with_capacity(16);
        let (_rx_bytes_count, _rx_host) = discover_socket.recv_from(&mut rx_discover_buffer).unwrap();
        //rpProMi:App,0161\n\n
    };
}
