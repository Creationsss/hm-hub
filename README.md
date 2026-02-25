# hm-hub

CLI tool for the HM Lab Z-NEO 8K USB Hub. Controls the built-in 320x170 LCD screen and reads power stats without needing the browser-based web app.

## Features

- Upload images and animated GIFs to the LCD display
- Read and set device config (brightness, rotation, etc.)
- Monitor USB power and per-port current draw
- Backup and restore device state
- Watch a directory and auto-upload when images change
- Auto-detects the device serial port

## Install

```
cargo install --path .
```

Or build manually:

```
cargo build --release
```

## Usage

The device is auto-detected. Use `-p /dev/ttyACMx` to override.

```
hm-hub info
hm-hub config
hm-hub config set brightness 20
hm-hub config set rotation 90
hm-hub upload photo.png
hm-hub upload image1.jpg image2.png animation.gif
hm-hub slideshow ./my-images/
hm-hub power
hm-hub power --watch
hm-hub monitor
hm-hub read -o ./output/
hm-hub backup device.bak
hm-hub restore device.bak
hm-hub rotate ./my-images/ --interval 300
hm-hub reset
```

Run `hm-hub config set` with no arguments to see all available config fields.

## License

AGPL-3.0-or-later
