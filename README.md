# Rust library for Novastar LED Screen processors
Contains the core functions for interacting with Novastar LED Screen processors

Note only the following functions have been implemented so far;
- Set global brightness
- Convert hardware names from presented IDs

highest priority todo
- Implement NET interface
- Test cascaded controllers on serial 

### Usage
```
use novastar_core;

novastar_core::discover();
let controllers = novastar_core::get_controllers();

for i in 0..255 {
    if controllers.len() > 0 {
        for controller in controllers {
            controller.set_brightness(i);
        }
    }
    sleep(1);
}

```