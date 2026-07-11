#include "wavegen.c"
#include <stdio.h>
#include <string.h>

static uint8_t wave_buffer[BUFFER_LEN]  __attribute__ ((aligned (8))) = {
    0
};

int main() {
    uint8_t img[DISPLAY_HEIGHT + DISPLAY_WIDTH * DISPLAY_HEIGHT * COLOR_DEPTH * NUM_DISPLAYS / 8 + DISPLAY_WIDTH] = {0};
    memset(img, 0xFF, sizeof(img));

    int len = build_wave(img, wave_buffer);
    //int len = 3;
    printf("buffer len: %d\n", BUFFER_LEN);
    printf("actual len: %d\n", len);

    return 0;
}
