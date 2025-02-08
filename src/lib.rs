mod novastarpacket;
mod controller;
mod net;
mod types;
use std::{io::Error, thread::sleep, time::Duration};
use novastarpacket::{build_tx_sender, NovastarPacket};
use types::{FeatureAddress, OpCode};

use crate::controller::*;

pub struct NovastarClient {}

pub fn discover() {
  tokio::spawn(discover_loop());
}
async fn discover_loop() {
  loop {
    net::discover();
    com_discover();
    sleep(Duration::from_secs(30));
  }
}
/*
pub fn get_controllers() -> Vec<&Controller> {
  return controller::get_controllers();
}*/

pub fn get_controllers() -> &'static mut Vec<Controller> {
  return controller::get_controllers();
}

fn print_bytes(prefix: &str, buf: &[u8]) {
  print!("{prefix} Content: ");
  for i in buf {
      print!("{:01$x} ", i, 2);
  }
  println!();
}

fn com_discover() {
    let _ = match serialport::available_ports() {
        Ok(ports) => {
            for port_info in ports {
                let port_name = port_info.port_name.as_str();
                println!("{port_name}");
                  match get_controller(port_name.to_string()) {
                    Some(_) => println!("Port Already Associated"),
                    None => { 
                      match try_com_connect(port_name, 1048576) {
                        Ok(_) => (),
                        Err(_) => {
                          let _ = try_com_connect(port_name, 115200);
                        }
                      } 
                    },
                };
            }
        },
        Err(e) => println!("Failed to discover ports {e}"),
    };
}

fn try_com_connect(port_name: &str, baud_rate: u32) -> Result<(), Error> {
  let _ = match serialport::new(port_name, baud_rate).open() {
    Ok(mut port) => {
        let _ = port.set_timeout(Duration::from_secs(1));
        let mut data: Vec<u8> = Vec::new();
        data.push(0);
        data.push(0);
        let tx_buff = build_tx_sender(OpCode::Read, 0x00, FeatureAddress::ControllerModelIdAddr, data);
        match port.write(&tx_buff) {
          Ok(_) => {
            let _ = port.flush();
            let rx_buff: &mut [u8; 22] = &mut [0; 22];
            match port.read_exact(rx_buff) {
              Ok(_) => {
                match NovastarPacket::decode(rx_buff.to_vec()) {
                  Ok(np) => {
                    let dev_id = u16::from_le_bytes([np.data[0], np.data[1]]);
                    add_serial_controller(port_name.to_string(),dev_id, port);
                  },
                  Err(_np) => println!("Not Novastar Hardware"),
                };
              },
              Err(e) => { println!("Serial Read Error {e}"); return Err(e); },
            }
          },
          Err(e) => println!("Failed to write to serial {e}"),
        }
    },
    Err(e) => println!("Serial Open Failed {e}"),
  };
  return Ok(());
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