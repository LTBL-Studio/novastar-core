#![allow(dead_code)]

use crate::{print_bytes, types::*};
use std::io::Write;
use num_enum::TryFromPrimitive;

static mut PACKET_SERIAL: u8 = 0;
static MY_ADDR: u8 = 0xFE;

#[derive(Debug)]
pub struct NovastarPacket {
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
    pub data: Vec<u8>,
}

impl NovastarPacket {
    fn encode(self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(0);
        let _ = out.write(&self.direction.to_be_bytes());
        out.push(self.ack);
        out.push(self.serial);
        out.push(self.src_addr);
        out.push(self.dst_addr);
        out.push(self.device_type as u8);
        out.push(self.port_addr);
        let _ = out.write(&self.scanboard_addr.to_le_bytes());
        out.push(self.op_code as u8);
        out.push(self.reserved2);
        let _ = out.write(&(self.address as u32).to_le_bytes());
        let _ = out.write(&(self.data.len() as u16).to_le_bytes());
        let mut check_len = 18;
        if self.op_code == OpCode::Write {
            let _ = out.write(&self.data);
            check_len += self.data.len();
        }
        let mut checksum: u16 = 0x5555;
        for i in 2..check_len {
            checksum += out[i] as u16;
        }
        let _ = out.write(&checksum.to_le_bytes());
        unsafe {
            PACKET_SERIAL += 1;
            if PACKET_SERIAL == 255 {PACKET_SERIAL = 0; }
        }
        print_bytes("encode: ", &out);
        return out;
    }
    pub fn decode(buff: Vec<u8>) -> Result<NovastarPacket, NovastarPacket> {
        print_bytes("decode: ", &buff);
        let data_len: u16 = u16::from_le_bytes([buff[16], buff[17]]);
        let out: NovastarPacket = NovastarPacket {
            direction: u16::from_be_bytes([buff[0], buff[1]]),
            ack: buff[2],
            serial: buff[3],
            src_addr: buff[4],
            dst_addr: buff[5],
            device_type: DeviceType::try_from_primitive(buff[6]).unwrap(),
            port_addr: buff[7],
            scanboard_addr: u16::from_be_bytes([buff[8], buff[9]]),
            op_code: OpCode::try_from_primitive(buff[10]).unwrap(),
            reserved2: 0x00,
            address:  FeatureAddress::try_from_primitive(u32::from_le_bytes([buff[12], buff[13], buff[14], buff[15]])).unwrap(),
            data: buff[18..data_len as usize + 18].to_vec(),
        };
        let rx_checksum: u16 = u16::from_le_bytes([buff[18 + data_len as usize], buff[19 + data_len as usize]]);
        let mut checksum: u16 = 0x5555;
        for i in 2..(out.data.len() + 18) {
            checksum += buff[i] as u16;
        }
        if rx_checksum == checksum {
            return Ok(out);
        } else {
            println!("rx_checksum: {}",rx_checksum);
            println!("calc_checksum: {}",checksum);
            return Err(out);
        }
    }
}


pub fn build_tx_sender(op_code: OpCode, dst_addr: u8, address: FeatureAddress, data: Vec<u8>) -> Vec<u8> {
    let my_serial: u8;
    unsafe { my_serial = PACKET_SERIAL; }
    let out: NovastarPacket = NovastarPacket {
        direction: 0x55AA,
        ack: 0x00,
        serial: my_serial,
        src_addr: MY_ADDR,
        dst_addr: dst_addr,
        device_type: DeviceType::Controller,
        port_addr: 0x00,
        scanboard_addr: 0x0000,
        op_code: op_code,
        reserved2: 0x00,
        address: address,
        data: data,
    };
    return out.encode();
}

pub fn build_tx_scanboard(op_code: OpCode, address: FeatureAddress, data: Vec<u8>) -> Vec<u8> {
    let my_serial: u8;
    unsafe { my_serial = PACKET_SERIAL; }
    let out: NovastarPacket = NovastarPacket {
        direction: 0x55AA,
        ack: 0x00,
        serial: my_serial,
        src_addr: MY_ADDR,
        dst_addr: 0xFF,
        device_type: DeviceType::Scanboard,
        port_addr: 0xFF,
        scanboard_addr: 0xFFFF,
        op_code: op_code,
        reserved2: 0x00,
        address: address,
        data: data,
    };
    return out.encode();
}
