# NOLP Terminal

A serial terminal that uses a Text User Interface (TUI) to allow the application to be run in a terminal emulator.

## Features

![Menu View](resources/menu-view.png)

### Encoding

The communication to and from the serial port can be encoded with the following formats:

- ASCII
- Decimal
- Hex
- Octal

### Keymaps

Movement is based on keyboard input:

**Display Keymaps:**\
*Ctrl + n* &nbsp;&nbsp; Displays the menu view\
*Ctrl + l* &nbsp; &nbsp; Displays the device list view\
*Ctrl + q* &nbsp;&nbsp; Quits the application

**Movement Keymaps:**\
*Ctrl + [* &nbsp;&nbsp; Selects the previous element\
*Ctrl + ]* &nbsp;&nbsp; Selects the next element

>[!NOTE]
> At certain breakpoints, the application will render a scrollbar. The `movement keymaps` will control the scroll in this case.

## System Requirements

Rust version 1.71.0 or greater

There may be some dependencies that are related to `serialport-rs` as this is used for the serial port communication. Please refer to https://github.com/serialport/serialport-rs#dependencies for those requirements.

## Building from Source

The build should be as simple as using `cargo build`
