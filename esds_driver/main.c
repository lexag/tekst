#include <stdio.h>
#include <math.h>
#include "pico/stdlib.h"
#include "hardware/pio.h"
#include "hardware/dma.h"
#include "hardware/clocks.h"

// This header is generated from pinstream.pio
#include "pinstream.pio.h"

// ===== CONFIG =====
#define PIN_BASE 6
#define PIN_COUNT 6

#define CLK 6 - PIN_BASE
#define BLANK 7 - PIN_BASE
#define DATA_GREEN 8 - PIN_BASE
#define LATCH 9 - PIN_BASE
#define DATA_RED 10 - PIN_BASE
#define CLK2 11 - PIN_BASE

#define PIN_DEBUG 25


#define DISPLAY_HEIGHT 16
#define DISPLAY_WIDTH 512
#define SCAN_ROWS 17
#define RESET_ROW 0
// Example waveform buffer (bit patterns for pins)
#define BRIGHTNESS_RANGE 16000
#define HEADER_LEN 173
#define DATA_TICK_LEN 8
#define PAUSE_LEN 10
#define DATA_LEN DISPLAY_WIDTH * DATA_TICK_LEN + PAUSE_LEN * DISPLAY_WIDTH / 8
#define PACKET_LEN HEADER_LEN + DATA_LEN 
//#define PACKET_LEN BIT_LEN * WIDTH 
#define BUFFER_LEN PACKET_LEN
static uint32_t wave_buffer[BUFFER_LEN]  __attribute__ ((aligned (32))) = {
    0
};

static uint32_t state = 1u << LATCH;
static int wp = 0;
static int buf_sel = 0;
static bool wave_dirty = 0;

static int debug_val = 0;

static int dma_chan_data;

static int row_index = 0;



static inline void set_pin(int pin, int set) {
    state &= ~(1U << pin);
    state |= (set & 1U) << pin;
}

static inline void clk(int set) {
    set_pin(CLK, set);
    set_pin(CLK2, set);
}

void write(int ticks) {
    for (int i = 0; i < ticks; i++) {
        if (wp < 0 || wp >= BUFFER_LEN) {
            return;
        }
        wave_buffer[wp] = state;
        wp++;
    }
    
}

void data_tick(int idx, int dat_r, int dat_g, int brightness) {
    clk(1);
    set_pin(DATA_GREEN, !dat_g);
    set_pin(DATA_RED, !dat_r);
    write(5);

    if (idx == (DISPLAY_WIDTH * (255 - ((brightness - 2)/2 + 128))) / 255 - 1) {
        set_pin(BLANK, 0);
    }
    
    clk(0);
    if (idx % 8 == 7) {
        write(PAUSE_LEN);
    }
    write(10);
}


void header(bool no_blank) {
    clk(1);
    set_pin(DATA_GREEN, 0);
    set_pin(DATA_RED, 0);
    write(29);

    if (!no_blank) set_pin(BLANK, 1);
    write(3); ;

    set_pin(LATCH, 0);
    write(3);

    set_pin(LATCH, 1);
    write(3);
    
    //if (!no_blank) set_pin(BLANK, 0);
    write(69);

    clk(0);
    set_pin(DATA_GREEN, 1);
    set_pin(DATA_RED, 1);
    write(10);
}

void wait_until_end() {
    write(BUFFER_LEN - wp);
}

void build_wave() {
    wp = 0;
    header(row_index == RESET_ROW);
    for (int x = 0; x < DISPLAY_WIDTH; x++) {
        data_tick(x, x/2 % 2 == 0, (1+x)/2 % 2 == 1, 255);
    }

    row_index++;
    row_index %= SCAN_ROWS;
}


void dma_irq_handler() {
    dma_hw->ints0 = 1u << dma_chan_data;

    dma_channel_acknowledge_irq0(dma_chan_data);
    
    dma_channel_wait_for_finish_blocking(dma_chan_data);
    dma_channel_set_read_addr(dma_chan_data, wave_buffer, false);
    dma_channel_set_trans_count(dma_chan_data, wp, false);
    dma_channel_start(dma_chan_data);

    build_wave();
}

int main() {
    build_wave();

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
    sm_config_set_out_shift(&c, false, true, 32);

    // FIFO join for TX (important for streaming)
    sm_config_set_fifo_join(&c, PIO_FIFO_JOIN_TX);

    // Set clock divider (adjust speed here)
    float div = 6.00f;
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
    //uint dma_chan_ctrl = dma_claim_unused_channel( true );
    
    dma_channel_config dma_chan_config_data = dma_channel_get_default_config( dma_chan_data );
    //dma_channel_config dma_chan_config_ctrl = dma_channel_get_default_config( dma_chan_ctrl );
    

    channel_config_set_transfer_data_size( &dma_chan_config_data, DMA_SIZE_32 );
    channel_config_set_read_increment( &dma_chan_config_data, true );
    channel_config_set_write_increment( &dma_chan_config_data, false );
    channel_config_set_dreq( &dma_chan_config_data, pio_get_dreq(pio, sm, true) );
    //channel_config_set_chain_to(&dma_chan_config_data, dma_chan_ctrl);
    //channel_config_set_ring( &dma_chan_config_data, false, __builtin_ctz(BUFFER_LEN * 4));
    dma_channel_configure(
        dma_chan_data,
        &dma_chan_config_data,
        &pio->txf[sm],
        wave_buffer,    
        wp,      
        true
    );


    //const int len = BUFFER_LEN;
    //const int* len_ptr = &len;

    //channel_config_set_transfer_data_size( &dma_chan_config_ctrl, DMA_SIZE_32 );
    //channel_config_set_read_increment( &dma_chan_config_ctrl, false );
    //channel_config_set_write_increment( &dma_chan_config_ctrl, false );
    //channel_config_set_dreq( &dma_chan_config_ctrl, 0x3f ); // 0x3f = no pacing = as fast as possible
    //channel_config_set_chain_to( &dma_chan_config_ctrl, dma_chan_data );
    //dma_channel_configure(
    //    dma_chan_ctrl,
    //    &dma_chan_config_ctrl,
    //    &(dma_channel_hw_addr(dma_chan_data)->transfer_count),
    //    len_ptr,
    //    1,      
    //    true
    //);
    //dma_channel_start(dma_chan_data);



    // //dma_channel_config_t dma_c_a = dma_transfer_init(pio, sm, dma_chan_data, dma_chan_ctrl);
    // //dma_channel_config_t dma_c_b = dma_transfer_init(pio, sm, dma_chan_ctrl, dma_chan_data);

    // dma_channel_start(dma_chan_data);

    gpio_put(PIN_DEBUG, 1);

    

    //dma_chan= dma_claim_unused_channel(true);
    //dma_channel_config dc = dma_channel_get_default_config(dma_chan);

    //channel_config_set_transfer_data_size(&dc, DMA_SIZE_32);
    //channel_config_set_read_increment(&dc, true);
    //channel_config_set_write_increment(&dc, false);

    //// Ring buffer: wraps at BUFFER_LEN * 4 bytes
    ////channel_config_set_ring(&dc, true, log2(BUFFER_LEN * 4)); 
    //// 2^5 = 32 bytes ring size (must be >= buffer size in power of 2)

    //// Write address = PIO TX FIFO
    //dma_channel_configure(
    //    dma_chan,
    //    &dc,
    //    &pio->txf[sm],      // write to PIO TX FIFO
    //    wave_buffer,        // read from waveform buffer
    //    BUFFER_LEN,         // unlimited transfers (ring handles looping)
    //    true               // start immediately
    //);

    dma_channel_set_irq0_enabled(dma_chan_data, true);
    

    irq_set_exclusive_handler(DMA_IRQ_0, dma_irq_handler);
    irq_set_enabled(DMA_IRQ_0, true);

    //
    //while (true) {
    //    tight_loop_contents();
    //}
    
    //while (true) {
    //    pio_sm_put_blocking(pio, sm, 0xFFFFFFFF);
    //    sleep_ms(500);
    //
    //    pio_sm_put_blocking(pio, sm, 0x00000000);
    //    sleep_ms(500);
    //}

    while (true) {
        tight_loop_contents();
    }
}

