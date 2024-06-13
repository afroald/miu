# miu-com

A small utility for controlling the main instrument unit (MIU) from a Saab 9-5.

The reason for making this is that I want to swap a Saab B235 engine into another car, and for that I have to make a new instrument cluster that receives information from the Trionic 7 ECU. Developing this application was very useful for figuring out which CAN messages are important and what they mean. This application will aid in the development of the new instrument cluster because I can use it to simulate parts of the actual car.

This utility is so specific that it's probably not useful for anyone. This readme is mostly a reminder to myself. Note that the error handling is pretty basic because this is not production software and I preferred to spend my time on other things.

## Requirements

- This application uses [SocketCAN](https://www.kernel.org/doc/html/v4.17/networking/can.html) so only Linux is supported for now.
- It is possible to test the application using a _virtual_ CAN interface.

## Configuring the CAN interface

Run the following commands to configure and start the interface. I use a [Canable Pro](https://canable.io) personally. YMMV.

```sh
sudo ip link set can0 type can bitrate 500000 restart-ms 100
sudo ip link set up can0
```

Or configure a virtual interface:

```sh
sudo modprobe vcan # (if needed)
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0
```

## Getting Started

- Configure the CAN interface as described above
- Start the application: `cargo run`
- If you use a virtual interface you might want to set the log level to `debug` to get an idea of what message are sent on the bus: `RUST_LOG=debug cargo run`
