cmake_minimum_required(VERSION 3.13)

set(PICO_SDK_PATH ${CMAKE_CURRENT_LIST_DIR}/pico-sdk)
include(${PICO_SDK_PATH}/external/pico_sdk_import.cmake)

project(pio_dma_stream C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

pico_sdk_init()

add_executable(pio_dma_stream
    main.c
    pinstream.pio
)

pico_generate_pio_header(pio_dma_stream ${CMAKE_CURRENT_LIST_DIR}/pinstream.pio)

target_link_libraries(pio_dma_stream
    pico_stdlib
    hardware_pio
    hardware_dma
    hardware_i2c
)

pico_add_extra_outputs(pio_dma_stream)
