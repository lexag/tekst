#include <stdint.h>
#define PIN_BASE 5
#define PIN_COUNT 8

const int display_pins[10] = {5 - PIN_BASE, 6 - PIN_BASE, 8 - PIN_BASE, 7 - PIN_BASE, 9 - PIN_BASE, 10 - PIN_BASE, 11 - PIN_BASE, 12 - PIN_BASE, 7 - PIN_BASE, 9 - PIN_BASE};
const uint8_t display_pin_masks[10] = {
    1 << display_pins[0],
    1 << display_pins[1],
    1 << display_pins[2],
    1 << display_pins[3],
    1 << display_pins[4],
    1 << display_pins[5],
    1 << display_pins[6],
    1 << display_pins[7],
    1 << display_pins[8],
    1 << display_pins[9],
};

static int pin_address_offset = 0;
int pin(int pin) {
    return display_pins[pin + pin_address_offset];
}

static inline int pin_mask(int pin) {
    return display_pin_masks[pin];
}
