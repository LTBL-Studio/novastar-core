#![warn(missing_docs)]
//! novastar-core is a crate used for interacting with Novastar LED Screen processors

pub mod net;
pub mod serial;

mod controller;
mod novastarpacket;
mod types;

use crate::controller::*;

/// Returns an Iterator over the Novastar [Controller]s available on the network and on serial ports of the machine
///
/// See [net::discover] and [serial::dicover] for specific discover
pub fn discover() -> Result<impl Iterator<Item = Controller>, std::io::Error> {
    Ok(net::discover()?.chain(serial::discover()?))
}

#[cfg(feature = "debug")]
fn print_bytes(prefix: &str, buf: &[u8]) {
    print!("{prefix} Content: ");
    for i in buf {
        print!("{:01$x} ", i, 2);
    }
    println!();
}
