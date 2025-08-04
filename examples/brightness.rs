use novastar_core::{net, serial};

fn main() {
    if let Ok(controllers) = net::discover() {
        for mut controller in controllers {
            println!("Hosts {controller:?}");
            println!("Brightness: {:?}", controller.brightness());
            controller.set_brightness(128).unwrap()
        }
    }

    if let Ok(controllers) = serial::discover() {
        for mut controller in controllers {
            println!("Hosts {controller:?}");
            controller.set_brightness(128).unwrap()
        }
    }
}
