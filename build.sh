#!/bin/bash

set -x
set -e

BUILD_NAME=myclockfirmware
# export DEFMT_LOG=info
export DEFMT_LOG=warn

# cargo build
# arm-none-eabi-objdump -x target/thumbv7m-none-eabi/debug/${BUILD_NAME} > target/thumbv7m-none-eabi/debug/${BUILD_NAME}.dump
# arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/debug/${BUILD_NAME}  target/thumbv7m-none-eabi/debug/${BUILD_NAME}.bin
# cp target/thumbv7m-none-eabi/debug/${BUILD_NAME}.bin /mnt/c/Users/moesa/Downloads/stm32flash-0.7-binaries/

cargo build --release
arm-none-eabi-objdump -x target/thumbv7m-none-eabi/release/${BUILD_NAME} > target/thumbv7m-none-eabi/release/${BUILD_NAME}.dump
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/${BUILD_NAME}  target/thumbv7m-none-eabi/release/${BUILD_NAME}.bin
cp target/thumbv7m-none-eabi/release/${BUILD_NAME}.bin /mnt/c/Users/moesa/Downloads/stm32flash-0.7-binaries/
cp target/thumbv7m-none-eabi/release/${BUILD_NAME} /mnt/c/Users/moesa/Downloads/stm32flash-0.7-binaries/
