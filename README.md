# NOLP Terminal

A serial terminal that uses a Text User Interface (TUI) to allow the application to be run in a terminal emulator.

> [!NOTE]
> This project under active development and may have some bugs or may be lacking features that you need. If you find this is true for your use case, please submit a ticket and I will work to get the issue resolved or feature implemented as time permits.

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

If you run into trouble, there may be some dependencies that your system is missing related to the crates used in this project. Please refer to the `System Requirements` section.


## Contributing

If you would like to contribute, by all means please do so. If you need any help getting started contributing, please take out a ticket and I would be happy to assist.

## Looking Ahead

- `custom keymaps` Adding a config file in the root of the directory to control this may be a simple yet effective approach.
- `additional settings` This may include a secondary menu view. Potential for features like new line toggle.
- `mouse input` Not sure the effort involved in this, it may be simple or a little more involved.