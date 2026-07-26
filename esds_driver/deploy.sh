#!/bin/bash
set -e

export PICO_SDK_PATH=$(pwd)/pico-sdk

rm -rf build
mkdir build
cd build

cmake ..
make -j$(nproc)

cd ..

picotool load build/tekst_driver.elf -f
picotool reboot
