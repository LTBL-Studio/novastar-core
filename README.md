# Rust library for Novastar LED Screen processors
Contains the core functions for interacting with Novastar LED Screen processors

Note only the following functions have been implemented so far;
- Set global brightness
- Convert hardware names from presented IDs

highest priority todo
- Test cascaded controllers on serial
- Test brightness getter on more controllers

### Usage
```
use novastar_core;

novastar_core::discover();

if let Ok(controllers) = net::discover() {
  for mut controller in controllers {
    println!("Hosts {controller:?}");
    controller.set_brightness(128).unwrap()
  }
}

```
