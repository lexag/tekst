#include <stdio.h>
#include <math.h>
#include <string.h>
#include "pico/stdlib.h"
#include "hardware/pio.h"
#include "hardware/dma.h"
#include "hardware/clocks.h"
//#include "hardware/spi.h"
#include "hardware/i2c.h"

// This header is generated from pinstream.pio
#include "pinstream.pio.h"

#include "wavegen.c"

// ===== CONFIG =====

// clk, blank, latch, data_r, data_g
#define PIN_SDA 22
#define PIN_SCL 23
#define I2C0_PERIPHERAL_ADDR 0x30

#define PIN_DEBUG 25

static uint8_t wave_buffer[BUFFER_LEN]  __attribute__ ((aligned (8))) = {
    0
};
static uint8_t rx_buffer[DISPLAY_HEIGHT + DISPLAY_WIDTH * DISPLAY_HEIGHT * COLOR_DEPTH * NUM_DISPLAYS / 8 + DISPLAY_WIDTH] = {
    0xFF
};
static uint32_t rx_index = 0;
static uint8_t rx_ident = 0;

static volatile bool wave_dirty = 0;
static int row_index = 0;
static int actual_buffer_len = 0;

static int brightness_ptr = 0;

static int debug_val = 0;


static int dma_chan_data;

void flip_led() {
    gpio_put(PIN_DEBUG, debug_val);
    debug_val = !debug_val;
}




void __isr i2c1_irq_handler(void)
{
    i2c_hw_t *hw = i2c_get_hw(i2c1);


    uint32_t status = hw->raw_intr_stat;

    // Master wrote data to us
    if (status & I2C_IC_RAW_INTR_STAT_RX_FULL_BITS)
    {
        while (hw->status & I2C_IC_STATUS_RFNE_BITS)
        {
            uint8_t b = (uint8_t)hw->data_cmd;

            if (rx_index < sizeof(rx_buffer))
                rx_buffer[rx_index++] = b;
        }
    
    }
    // STOP condition = transfer complete
    if (status & I2C_IC_RAW_INTR_STAT_STOP_DET_BITS)
    {
        hw->clr_stop_det;
        rx_ident += 1;
        //flip_led();

        if (rx_ident == 3) {
            actual_buffer_len = build_wave(rx_buffer, wave_buffer);
            flip_led();
            //flip_led();
            rx_index = 0;
            rx_ident = 0;
            wave_dirty = true;
        }
    }
}
void dma_irq_handler() {
    dma_hw->ints0 = 1u << dma_chan_data;

    dma_channel_acknowledge_irq0(dma_chan_data);
    
    dma_channel_wait_for_finish_blocking(dma_chan_data);
    dma_channel_set_read_addr(dma_chan_data, wave_buffer, false);
    dma_channel_set_trans_count(dma_chan_data, actual_buffer_len, false);
    dma_channel_start(dma_chan_data);

    if (wave_dirty) {
        //build_wave();
        wave_dirty = false;
    }
}

int main() {
    memset(rx_buffer, 0xFF, sizeof(rx_buffer));
    actual_buffer_len = build_wave(rx_buffer, wave_buffer);

    stdio_init_all();

    gpio_init(PIN_DEBUG);
    gpio_set_dir(PIN_DEBUG, GPIO_OUT);


    // ===== PIO SETUP =====
    PIO pio = pio0;
    uint sm = 0;

    uint offset = pio_add_program(pio, &pinstream_program);

    pio_sm_config c = pinstream_program_get_default_config(offset);

    // Map OUT to GPIO pins
    sm_config_set_out_pins(&c, PIN_BASE, PIN_COUNT);
    pio_sm_set_consecutive_pindirs(pio, sm, PIN_BASE, PIN_COUNT, true);

    // Shift configuration:
    // - shift_left = false => shift right (common for DMA-fed data)
    // - autopull enabled every 32 bits
    sm_config_set_out_shift(&c, false, true, 8);

    // FIFO join for TX (important for streaming)
    sm_config_set_fifo_join(&c, PIO_FIFO_JOIN_TX);

    // Set clock divider (adjust speed here)
    float div = 12.00f;
    sm_config_set_clkdiv(&c, div);

    // Init GPIO function to PIO
    for (int i = PIN_BASE; i < PIN_BASE + PIN_COUNT; i++) {
        pio_gpio_init(pio, i);
        gpio_set_function(i, GPIO_FUNC_PIO0);
    }

    pio_sm_init(pio, sm, offset, &c);
    pio_sm_set_enabled(pio, sm, true);

    // ===== DMA SETUP =====
    
    dma_chan_data = dma_claim_unused_channel( true );
    
    dma_channel_config dma_chan_config_data = dma_channel_get_default_config( dma_chan_data );
    

    channel_config_set_transfer_data_size( &dma_chan_config_data, DMA_SIZE_8 );
    channel_config_set_read_increment( &dma_chan_config_data, true );
    channel_config_set_write_increment( &dma_chan_config_data, false );
    channel_config_set_dreq( &dma_chan_config_data, pio_get_dreq(pio, sm, true) );
    dma_channel_configure(
        dma_chan_data,
        &dma_chan_config_data,
        &pio->txf[sm],
        wave_buffer,    
        actual_buffer_len,      
        true
    );

    dma_channel_set_irq0_enabled(dma_chan_data, true);
    irq_set_exclusive_handler(DMA_IRQ_0, dma_irq_handler);
    irq_set_enabled(DMA_IRQ_0, true);

    // === SPI SETUP ===
    //gpio_set_function(PIN_MISO, GPIO_FUNC_SPI);
    //gpio_set_function(PIN_MOSI, GPIO_FUNC_SPI);
    //gpio_set_function(PIN_SCK,  GPIO_FUNC_SPI);
    //gpio_set_function(PIN_CS,   GPIO_FUNC_SPI);
    //spi_init(SPI_PORT, 1000000);
    //spi_set_format(SPI_PORT, 8, SPI_CPOL_0, SPI_CPHA_0, SPI_MSB_FIRST);
    //spi_set_slave(SPI_PORT, true);

    //spi_get_hw(SPI_PORT)->imsc = SPI_SSPIMSC_RXIM_BITS;
    //irq_set_exclusive_handler(SPI0_IRQ, spi0_irq_handler);
    //irq_set_enabled(SPI0_IRQ, true);

    i2c_init(i2c1, 100000000);

    gpio_set_function(PIN_SDA, GPIO_FUNC_I2C);
    gpio_set_function(PIN_SCL, GPIO_FUNC_I2C);

    gpio_pull_up(PIN_SDA);
    gpio_pull_up(PIN_SCL);

    i2c_set_slave_mode(i2c1, true, 0x30);

    irq_set_exclusive_handler(I2C1_IRQ, i2c1_irq_handler);
    irq_set_enabled(I2C1_IRQ, true);

    // Enable the interrupts we care about
    i2c_get_hw(i2c1)->intr_mask =
        I2C_IC_INTR_MASK_M_RX_FULL_BITS |
        I2C_IC_INTR_MASK_M_STOP_DET_BITS;


    while (true) {
        tight_loop_contents();
        //if (wave_dirty) {
        //    flip_led();
        //    wave_dirty = false;
        //    rx_index = 0;
        //}
    }
}

