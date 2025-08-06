use num_enum::TryFromPrimitive;
use serialport::SerialPort;
use std::fmt::Display;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;
use thiserror::Error;

use crate::novastarpacket::*;
use crate::types::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("write error: {0}")]
    Write(std::io::Error),
    #[error("read error: {0}")]
    Read(std::io::Error),
    #[error("flush error: {0}")]
    Flush(std::io::Error),
    #[error("invalid packet: {0}")]
    PacketDecoding(#[from] PacketError),
    #[error("Failed to connect: {0}")]
    Connection(io::Error)
}

#[derive(Debug)]
pub enum ConnexionType {
    Tcp(SocketAddr, TcpStream),
    Serial(String, Box<dyn SerialPort>),
}

impl Display for ConnexionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnexionType::Tcp(socket_addr, _) => write!(f, "Tcp({socket_addr})"),
            ConnexionType::Serial(port_name, _) => write!(f, "Serial({port_name})"),
        }
    }
}

#[derive(Debug)]
pub struct Controller {
    pub(crate) card_type: SenderCardType,
    pub(crate) connexion: ConnexionType,
}

impl Controller {
    //pub fn update_last_seen(mut self) {
    //  self.last_seen = chrono::offset::Utc::now();
    //}

    pub fn card_type(&self) -> SenderCardType {
        self.card_type
    }

    pub fn connection(&self) -> &ConnexionType {
        &self.connexion
    }

    pub fn set_brightness(&mut self, value: u8) -> Result<(), Error> {
        let out: Vec<u8> = build_tx_scanboard(
            OpCode::Write,
            FeatureAddress::GlobalBrightnessAddr,
            &[value],
        );
        self.write_all(&out).map_err(Error::Write)
    }

    pub fn brightness(&mut self) -> Result<u8, Error> {
        self.write_all(&build_tx_sender(
            OpCode::Read,
            0,
            FeatureAddress::GlobalBrightnessAddr,
            &[0],
        ))
        .map_err(Error::Write)?;

        self.flush().map_err(Error::Flush)?;

        let rx_buff: &mut [u8; 21] = &mut [0; 21];
        self.read_exact(rx_buff).map_err(Error::Read)?;
        NovastarPacket::decode(rx_buff)
            .map(|packet| packet.data[0])
            .map_err(Error::PacketDecoding)
    }

    pub fn model_id_query(&mut self) -> Result<u16, Error> {
        self.write_all(&build_tx_sender(
            OpCode::Read,
            0,
            FeatureAddress::ControllerModelIdAddr,
            &[0, 0],
        ))
        .map_err(Error::Write)?;

        let rx_buff: &mut [u8; 22] = &mut [0; 22];
        self.read_exact(rx_buff).map_err(Error::Read)?;

        NovastarPacket::decode(rx_buff)
            .map(|packet| u16::from_le_bytes([packet.data[0], packet.data[1]]))
            .map_err(Error::PacketDecoding)
    }

    pub fn session_reset(&mut self) -> Result<(), Error> {
        self.write_all(&build_tx_sender(
            OpCode::Read,
            0xFF,
            FeatureAddress::ControllerModelIdAddr,
            &[0],
        ))
        .map_err(Error::Write)
    }

    /// Tries to connect to a Novastar [Controller] on the provided [SocketAddr]
    pub fn try_from_tcp_addr(addr: SocketAddr) -> Result<Self, Error> {
        let mut stream = TcpStream::connect(addr).map_err(Error::Connection)?;
        stream.set_read_timeout(Some(Duration::from_secs(1))).map_err(Error::Connection)?;
        let tx_buff = build_tx_sender(
            OpCode::Read,
            0x00,
            FeatureAddress::ControllerModelIdAddr,
            &[0, 0],
        );

        stream.write_all(&tx_buff).map_err(Error::Write)?;
        stream.flush().map_err(Error::Flush)?;
        let rx_buff: &mut [u8; 22] = &mut [0; 22];
        stream.read_exact(rx_buff).map_err(Error::Read)?;

        let packet = NovastarPacket::decode(rx_buff)?;

        let dev_id: u16 = u16::from_le_bytes([packet.data[0], packet.data[1]]);
        
        let dev_model =
            SenderCardType::try_from_primitive(dev_id).unwrap_or(SenderCardType::Unknown);
            
        #[cfg(feature = "debug")]
        if dev_model == SenderCardType::Unknown {
            println!("Controller returned model ID {dev_id}");
            println!("{:01$x} ", dev_id, 2);
        }

        Ok(Self {
            card_type: dev_model,
            connexion: crate::ConnexionType::Tcp(addr, stream),
        })
    }
}

impl Read for Controller {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.connexion {
            ConnexionType::Tcp(_, stream) => stream.read(buf),
            ConnexionType::Serial(_, serial_port) => serial_port.read(buf),
        }
    }
}
impl Write for Controller {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.connexion {
            ConnexionType::Tcp(_socket_addr, stream) => stream.write(buf),
            ConnexionType::Serial(_, serial_port) => serial_port.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.connexion {
            ConnexionType::Tcp(_, stream) => stream.flush(),
            ConnexionType::Serial(_, serial_port) => serial_port.flush(),
        }
    }
}
