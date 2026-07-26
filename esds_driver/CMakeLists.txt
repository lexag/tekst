cmake_minimum_required(VERSION 3.13)

set(PICO_SDK_PATH ${CMAKE_CURRENT_LIST_DIR}/pico-sdk)
include(${PICO_SDK_PATH}/external/pico_sdk_import.cmake)

project(pio_dma_stream C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

pico_sdk_init()

add_executable(tekst_driver
    main.c
    pinstream.pio
)

pico_generate_pio_header(tekst_driver ${CMAKE_CURRENT_LIST_DIR}/pinstream.pio)
pico_enable_stdio_usb(tekst_driver 1)
pico_enable_stdio_uart(tekst_driver 0)

target_link_libraries(tekst_driver
    pico_stdlib
    hardware_pio
    hardware_dma
    hardware_i2c
)

pico_add_extra_outputs(tekst_driver)
