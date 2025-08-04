use crate::types::*;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

static mut PACKET_SERIAL: u8 = 0;
static MY_ADDR: u8 = 0xFE;

#[derive(Debug)]
pub struct NovastarPacket<'a> {
    pub direction: u16,
    pub ack: u8,
    pub serial: u8,
    pub src_addr: u8,
    pub dst_addr: u8,
    pub device_type: DeviceType,
    pub port_addr: u8,
    pub scanboard_addr: u16,
    pub op_code: OpCode,
    pub reserved2: u8,
    pub address: FeatureAddress,
    pub data: &'a [u8],
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("unknown device type: {0}")]
    DeviceType(#[from] TryFromPrimitiveError<DeviceType>),
    #[error("unknown operation code: {0}")]
    OpCode(#[from] TryFromPrimitiveError<OpCode>),
    #[error("unknown feature address: {0}")]
    FeatureAddress(#[from] TryFromPrimitiveError<FeatureAddress>),
    #[error("invalid checksum {0}, should be {1}")]
    Checksum(u16, u16),
}

impl NovastarPacket<'_> {
    fn encode(self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(20);
        out.extend_from_slice(&self.direction.to_be_bytes()); // 55 aa
        out.push(self.ack); // 00
        out.push(self.serial); // 01
        out.push(self.src_addr); // fe
        out.push(self.dst_addr); // ff
        out.push(self.device_type as u8); // 01
        out.push(self.port_addr); // ff
        out.extend_from_slice(&self.scanboard_addr.to_le_bytes()); // ff ff
        out.push(self.op_code as u8); // 01
        out.push(self.reserved2); // 00
        out.extend_from_slice(&(self.address as u32).to_le_bytes()); // 01 00 00 02
        out.extend_from_slice(&(self.data.len() as u16).to_le_bytes()); // 01 00

        let mut check_len = 18;
        if self.op_code == OpCode::Write {
            out.extend_from_slice(self.data);
            check_len += self.data.len();
        }

        let mut checksum: u16 = 0x5555;
        for byte in out[2..check_len].iter() {
            checksum += *byte as u16;
        }

        out.extend_from_slice(&checksum.to_le_bytes());
        unsafe {
            PACKET_SERIAL += 1;
            if PACKET_SERIAL == 255 {
                PACKET_SERIAL = 0;
            }
        }

        #[cfg(feature = "debug")]
        crate::print_bytes("encode: ", &out);
        out
    }

    pub fn decode(buff: &[u8]) -> Result<NovastarPacket, PacketError> {
        #[cfg(feature = "debug")]
        crate::print_bytes("decode: ", buff);
        let data_len: u16 = u16::from_le_bytes([buff[16], buff[17]]);
        let out: NovastarPacket = NovastarPacket {
            direction: u16::from_be_bytes([buff[0], buff[1]]),
            ack: buff[2],
            serial: buff[3],
            src_addr: buff[4],
            dst_addr: buff[5],
            device_type: DeviceType::try_from_primitive(buff[6])?,
            port_addr: buff[7],
            scanboard_addr: u16::from_be_bytes([buff[8], buff[9]]),
            op_code: OpCode::try_from_primitive(buff[10])?,
            reserved2: 0x00,
            address: FeatureAddress::try_from_primitive(u32::from_le_bytes([
                buff[12], buff[13], buff[14], buff[15],
            ]))?,
            data: &buff[18..data_len as usize + 18],
        };
        let rx_checksum: u16 =
            u16::from_le_bytes([buff[18 + data_len as usize], buff[19 + data_len as usize]]);
        let mut checksum: u16 = 0x5555;

        for byte in buff[2..(out.data.len() + 18)].iter() {
            checksum += *byte as u16;
        }

        if rx_checksum == checksum {
            Ok(out)
        } else {
            eprintln!("rx_checksum: {rx_checksum}");
            eprintln!("calc_checksum: {checksum}");
            Err(PacketError::Checksum(rx_checksum, checksum))
        }
    }
}

pub fn build_tx_sender(
    op_code: OpCode,
    dst_addr: u8,
    address: FeatureAddress,
    data: &[u8],
) -> Vec<u8> {
    let my_serial: u8;
    unsafe {
        my_serial = PACKET_SERIAL;
    }
    let out: NovastarPacket = NovastarPacket {
        direction: 0x55AA,
        ack: 0x00,
        serial: my_serial,
        src_addr: MY_ADDR,
        dst_addr,
        device_type: DeviceType::Controller,
        port_addr: 0x00,
        scanboard_addr: 0x0000,
        op_code,
        reserved2: 0x00,
        address,
        data,
    };
    out.encode()
}

#[allow(dead_code)]
pub fn build_rx_sender(
    op_code: OpCode,
    dst_addr: u8,
    address: FeatureAddress,
    data: &[u8],
) -> Vec<u8> {
    let my_serial: u8;
    unsafe {
        my_serial = PACKET_SERIAL;
    }
    let out: NovastarPacket = NovastarPacket {
        direction: 0x55AA,
        ack: 0x00,
        serial: my_serial,
        src_addr: MY_ADDR,
        dst_addr,
        device_type: DeviceType::Controller,
        port_addr: 0x01,
        scanboard_addr: 0x0001,
        op_code,
        reserved2: 0x00,
        address,
        data,
    };
    out.encode()
}

pub fn build_tx_scanboard(op_code: OpCode, address: FeatureAddress, data: &[u8]) -> Vec<u8> {
    let my_serial: u8;
    unsafe {
        my_serial = PACKET_SERIAL;
    }
    let out: NovastarPacket = NovastarPacket {
        direction: 0x55AA,
        ack: 0x00,
        serial: my_serial,
        src_addr: MY_ADDR,
        dst_addr: 0xFF,
        device_type: DeviceType::Scanboard,
        port_addr: 0xFF,
        scanboard_addr: 0xFFFF,
        op_code,
        reserved2: 0x00,
        address,
        data,
    };
    out.encode()
}

// Set Brightness
// 0x55, 0xaa, 0x00, 0x00, 0xfe, 0xff, 0x01, 0xff, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x01, 0x00, val,
// ________________________________________________________________________XXXXXXXXXXXXXXXXXXXXXXX____________XXXXX____________
// 0x55, 0xaa, 0x00, 0x4d, 0xfe, 0x00, 0x01, 0x0d, 0xff, 0xff, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x01, 0x00, 0x74, 0x25, 0x59
// {direction}                                                              <---Feature Address-->             <Value>
// 55      AA     00       34      FE    00        01        00          FF             FF              01       00     01    00    00    02    01       00         66    F1 58
// {magic}{magic}{ack}{serial}{srcAdd}{dstAdd}{devType}{portAddr}{scanBoardAddrLow}{scanBoardAddrHigh}{code}{reserved2}{address             }{dataLen}{dataLenHigh}{data}{checksum}
// 55      AA     00       A9      FE    FF        00        00          00             00              01       00     00 00 00 01             01       00         00    FE 57
// 55      AA     00       02      FE    00        00        00          00             00              00       00     02 00 00 00             02       00               59 56
// 55      AA     00       31      FE    FF        00        00          00             00              01       00     00 00 00 01             01       00         00    86 57
// 55      aa     00       01      fe    00        00        00          00             00              00       00     02 00 00 00             00       00               56 56
