# miu-com

https://github.com/crossbeam-rs/crossbeam

## Features
- Start/stop bus connection
- Mode: display/control

## Signals
- Engine speed (rpm)
- Vehicle speed (km/h)
- Boost (0-100%)
- Coolant temperature (°C)
- Fuel level (L)
- Check engine
- Check gearbox
- Selected gear
- Selected gear fault

## Display mode
Display data received from all devices on the bus, including the MIU.

- Receive: Engine speed (rpm)
- Receive: Vehicle speed (km/h)
- Receive: Boost (0-100%)
- Receive: Coolant temperature (°C)
- Receive: Fuel level (L)
- Receive: Check engine
- Receive: Check gearbox
- Receive: Selected gear
- Receive: Selected gear fault

## Control mode
Control the MIU in isolation by emulating devices connected to the bus for testing purposes.

- Send: Engine speed (rpm)
- Inactive: Vehicle speed (km/h)
- Send: Boost (0-100%)
- Send: Coolant temperature (°C)
- Inactive: Fuel level (L)
- Send: Check engine
- Send: Check gearbox
- Send: Selected gear
- Send: Selected gear fault

# Useful stuff

- https://docs.rs/nix/0.19.1/nix/ifaddrs/fn.getifaddrs.html
- https://docs.rs/interfaces/0.0.8/interfaces/struct.Interface.html
