#define PIN_BASE 5
#define PIN_COUNT 8

const uint32_t display_pins[10] = {5 - PIN_BASE, 6 - PIN_BASE, 8 - PIN_BASE, 7 - PIN_BASE, 9 - PIN_BASE, 10 - PIN_BASE, 11 - PIN_BASE, 12 - PIN_BASE, 7 - PIN_BASE, 9 - PIN_BASE};

static int pin_address_offset = 0;
int pin(int pin) {
    return display_pins[pin + pin_address_offset];
}
