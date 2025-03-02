#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(static_mut_refs)]

use std::fmt::Error;
use std::time::Duration;
use serialport::SerialPort;

use crate::types::*;
use crate::novastarpacket::*;

static mut CONTROLLERS: Vec<Controller> = Vec::new();

#[derive(Debug)]
pub struct Controller {
  pub port_name: String,
  pub socket: Option<std::net::SocketAddr>,
  pub model_id: SenderCardType,
  serial_port: Option<Box<dyn SerialPort>>,
  //pub last_seen: chrono::DateTime<Utc>,
}

impl Controller {
  //pub fn update_last_seen(mut self) {
  //  self.last_seen = chrono::offset::Utc::now();
  //}

    pub fn set_brightness(&mut self, value: u8) {
        let mut data: Vec<u8> = Vec::new();
        data.push(value);
        let out:  Vec<u8> = build_tx_scanboard(OpCode::Write, FeatureAddress::GlobalBrightnessAddr, data);
        self.write(out);
    }

    pub fn model_id_query(&mut self, mut out: NovastarPacket) -> Result<u16, Error> {
        let mut data: Vec<u8> = Vec::new();
        data.push(0);
        data.push(0);
        self.write(build_tx_sender(OpCode::Read, 0x00, FeatureAddress::ControllerModelIdAddr, data));
        let rx_buff: &mut [u8; 22] = &mut [0; 22];
        out = NovastarPacket::decode(self.read(rx_buff)).unwrap();
        return Ok(u16::from_le_bytes([out.data[0], out.data[1]]));
    }

    pub fn session_reset(&mut self) {
        let mut data: Vec<u8> = Vec::with_capacity(0);
        data.push(0);
        self.write(build_tx_sender(OpCode::Read, 0xFF, FeatureAddress::ControllerModelIdAddr, data));
    }

    fn write(&mut self, tx_buf: Vec<u8>) {
        let _ = self.open_port();
        match self.socket {
            Some(_s) => {

            },
            None => {
                let port = self.serial_port.as_mut().unwrap();
                match port.write(tx_buf.as_ref()) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Serial Write Error {} {}", self.port_name, e);
                        unsafe {
                            let del_id= CONTROLLERS.iter().position(|r| r.port_name == self.port_name).unwrap();
                            CONTROLLERS.remove(del_id);
                        }
                    },
                }
                let _ = port.flush();
            }  
        };
    }
    fn read(&mut self, rx_buff: &mut [u8]) -> Vec<u8> {
        let _ = self.open_port();
        match self.socket {
            Some(_s) => {
                
            },
            None => {
                let port = self.serial_port.as_mut().unwrap();
                match port.read_exact(rx_buff) {
                    Ok(_) => return rx_buff.to_vec(),
                    Err(e) => println!("Serial Read Error {}", e),
                };
            },
        };
        let out: Vec<u8> = Vec::new();
        return out;
    }

    fn open_port(&mut self) -> Result<(),serialport::Error> {
        if self.serial_port.is_none() {
            let _ = match serialport::new(self.port_name.to_string(), 1048576).open() {
                Ok(mut port) => {
                    let _ = port.set_timeout(Duration::from_secs(1));
                    self.serial_port = Some(port); 
                    return Ok(());
                },
                Err(e) => return Err(e),
            };    
        }
        return Ok(());
    }
}
/*
pub fn get_controllers() -> Vec<&'static Controller> {
    //unsafe { return CONTROLLERS.iter().filter(|r|(r.last_seen - chrono::offset::Utc::now()).num_seconds() < 60).collect(); }
    unsafe { return CONTROLLERS.iter().collect(); }
}
    */
pub fn get_controllers() -> &'static mut Vec<Controller> {
    //unsafe { return CONTROLLERS.iter().filter(|r|(r.last_seen - chrono::offset::Utc::now()).num_seconds() < 60).collect(); }
    unsafe { return &mut CONTROLLERS; }
}
pub fn get_controller(port_name: String) -> Option<&'static Controller> {
    unsafe { return CONTROLLERS.iter().find(|r| r.port_name == port_name).map(|v| v); }
}
pub fn add_serial_controller(port_name: String, device_model: SenderCardType, serial_port: Box<dyn SerialPort>) {
    unsafe { CONTROLLERS.push(Controller{
        port_name: port_name,
        socket: None,
        model_id: device_model,
        serial_port: Some(serial_port),
    });}
}



