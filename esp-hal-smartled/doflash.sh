#!/bin/sh -xe

cargo espflash flash --monitor --baud=921600 --features=esp32c6 --example hello_rgb_async
