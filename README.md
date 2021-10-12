# Grabbert

A basic video downloader written in Rust.

Its purpose is to download videos from https://dumpert.nl. It is made merely to play a bit with Rust, inspired by a silly video I wanted to know how to download (of type `m3u8`, which Dumpert uses).

![Grabbert demo - Dumpert video downloader](docs/demo.png)

## Pre-requisites
- ffmpeg
- Linux/WSL (or build it yourself for Windows and macOS)

## Usage
```bash
$ cd bin/linux
$ ./grabbert
```

Example input: https://www.dumpert.nl/item/100011687_d47ffbc1
