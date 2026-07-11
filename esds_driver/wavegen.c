#include <stdbool.h>
#include <stdint.h>
#include "pins.c"

#define CLK 0
#define BLANK 1
#define LATCH 2
#define DATA_RED 3
#define DATA_GREEN 4

#define NUM_DISPLAYS 2
#define DISPLAY_HEIGHT 16
#define DISPLAY_WIDTH 448
#define COLOR_DEPTH 2
#define SCAN_ROWS 17
#define RESET_ROW 0


static uint32_t state = 1u << LATCH;
static uint32_t wp = 0;

#define HEADER_LEN 15
#define DATA_TICK_LEN 5
#define PAUSE_LEN 0
#define DATA_LEN (DISPLAY_WIDTH * DATA_TICK_LEN + PAUSE_LEN * DISPLAY_WIDTH / 8)
#define PACKET_LEN (HEADER_LEN + DATA_LEN)
#define BUFFER_LEN (PACKET_LEN * SCAN_ROWS * NUM_DISPLAYS)



static inline void set_pin(int pin, int set) {
    state &= ~(1U << pin);
    state |= (set & 1U) << pin;
}

static inline void clk(int set) {
    set_pin(pin(CLK), set);
}


void write(uint8_t out[], int ticks) {
    for (int i = 0; i < ticks; i++) {
        if (wp < 0 || wp >= BUFFER_LEN) {
            //gpio_put(PIN_DEBUG, 1);
            return;
        }
        out[wp] = state;
        wp++;
    }
    
}

void data_tick(uint8_t out[], int idx, int dat_r, int dat_g, int brightness) {
    clk(1);
    set_pin(pin(DATA_GREEN), !dat_g);
    set_pin(pin(DATA_RED), !dat_r);
    write(out, 2);

    //if (idx == (DISPLAY_WIDTH * (255 - ((brightness - 2)/2 + 128))) / 255 - 1) {
    //    set_pin(BLANK, 0);
    //}
    //if (idx == DISPLAY_WIDTH - brightness) {
    //    set_pin(BLANK, 0);
    //}
    
    if (idx == brightness) {
        set_pin(pin(BLANK), 0);
    }
    
    clk(0);
    // is this needed?
    //if (idx % 8 == 7) {
    //    write(out, PAUSE_LEN);
    //}
    write(out, 3);
}


void header(uint8_t out[], bool no_blank, int brightness) {
    clk(1);
    set_pin(pin(DATA_GREEN), 0);
    set_pin(pin(DATA_RED), 0);
    write(out, 3);

    if (!no_blank) {
        set_pin(pin(BLANK), 1);
    }
    write(out, 3); ;

    set_pin(pin(LATCH), 0);
    write(out, 3);

    set_pin(pin(LATCH), 1);
    write(out, 3);
    
    //set_pin(pin(BLANK), 0);
    //write(out, 10);

    clk(0);
    set_pin(pin(DATA_GREEN), 1);
    set_pin(pin(DATA_RED), 1);
    write(out, 3);
}

void wait_until_end(uint8_t out[]) {
    write(out, BUFFER_LEN - wp);
}


int build_wave(uint8_t img_buf[], uint8_t out[]) {
    wp = 0;
    const int PIXEL_BITS = DISPLAY_HEIGHT * DISPLAY_WIDTH / 8;
    const int px_start = DISPLAY_HEIGHT * NUM_DISPLAYS;

    for (int i = 0; i < DISPLAY_HEIGHT + 1; i++) {
        int row = DISPLAY_HEIGHT - 1 - (i % DISPLAY_HEIGHT);
        //int bright = rx_buffer[row];
        int bright = img_buf[0];
        int row_offs = row*DISPLAY_WIDTH / 8;

        for (int display_idx = 0; display_idx < NUM_DISPLAYS; display_idx++) {
            pin_address_offset = 5 * display_idx;
            int red_offs = px_start + display_idx * PIXEL_BITS;
            int green_offs = red_offs + PIXEL_BITS * NUM_DISPLAYS;

            set_pin(pin(BLANK), 1);
            for (int x = 0; x < DISPLAY_WIDTH; x++) {
                int px_offs = x/8 + row_offs;
                bool red = (img_buf[red_offs + px_offs] & (0x1 << (7 - (x % 8)))) > 0;
                bool green = (img_buf[green_offs + px_offs] & (0x1 << (7 - (x % 8)))) > 0;

                //if (x == bright * DISPLAY_WIDTH / 255) {
                //    pin_address_offset = 5 - 5 * display_idx;
                //    set_pin(pin(BLANK), 0);
                //}
                //pin_address_offset = 5 * display_idx;

                data_tick(out, x, red, green, bright);
            }
            header(out, false, 0);
        }

    };

    for (int display_idx = 0; display_idx < NUM_DISPLAYS; display_idx++) {
        pin_address_offset = 5 * display_idx;
        for (int x = 0; x < DISPLAY_WIDTH; x++) {
            data_tick(out, x, 0, 0, 255);
        }
        header(out, true, 0);
    }

    return wp;
}
