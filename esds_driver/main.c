#include "hardware/clocks.h"
#include "hardware/dma.h"
#include "hardware/pio.h"
#include "pico/stdlib.h"
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
// #include "hardware/spi.h"
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

static uint8_t wave_a[BUFFER_LEN] __attribute__((aligned(8))) = {0};
static uint8_t wave_b[BUFFER_LEN] __attribute__((aligned(8))) = {0};

static uint8_t (*read_wave)[BUFFER_LEN] = &wave_a;
static uint8_t (*write_wave)[BUFFER_LEN] = &wave_b;

static uint8_t
    rx_buffer[DISPLAY_HEIGHT +
              DISPLAY_WIDTH * DISPLAY_HEIGHT * COLOR_DEPTH * NUM_DISPLAYS / 8 +
              DISPLAY_WIDTH] = {0xFF};
static volatile uint32_t rx_index = 0;
static uint8_t rx_ident = 0;

static volatile bool wave_dirty = 0;
static volatile bool swap_pending = 0;
static int row_index = 0;
static volatile int actual_buffer_len = 0;

static int brightness_ptr = 0;

static int debug_val = 0;

static int dma_chan_data;

void flip_led() {
  gpio_put(PIN_DEBUG, debug_val);
  debug_val = !debug_val;
}

void __isr i2c1_irq_handler(void) {
  i2c_hw_t *hw = i2c_get_hw(i2c1);

  uint32_t status = hw->raw_intr_stat;

  // Master wrote data to us
  if (status & I2C_IC_RAW_INTR_STAT_RX_FULL_BITS) {
    while (hw->status & I2C_IC_STATUS_RFNE_BITS) {
      uint8_t b = (uint8_t)hw->data_cmd;

      if (rx_index < sizeof(rx_buffer))
        rx_buffer[rx_index++] = b;
    }
  }
  // STOP condition = transfer complete
  if (status & I2C_IC_RAW_INTR_STAT_STOP_DET_BITS) {
    hw->clr_stop_det;
    rx_ident += 1;
    //flip_led();

    if (rx_ident == 3) {
      restart_animation();
      wave_dirty = true;
      flip_led();
      rx_index = 0;
      rx_ident = 0;
    }
  }
}
void dma_irq_handler() {
  //dma_hw->ints0 = 1u << dma_chan_data;

  dma_channel_acknowledge_irq0(dma_chan_data);

  if (swap_pending) {
    uint8_t (*temp)[BUFFER_LEN] = read_wave;
    read_wave = write_wave;
    write_wave = temp;
    swap_pending = false;
  }

  //dma_channel_wait_for_finish_blocking(dma_chan_data);
  dma_channel_set_read_addr(dma_chan_data, *read_wave, false);
  dma_channel_set_trans_count(dma_chan_data, actual_buffer_len, false);
  dma_channel_start(dma_chan_data);
}

int new_buffer() {
  //flip_led();
  actual_buffer_len = build_wave(rx_buffer, *write_wave);
  swap_pending = true;
}

int init() {
  printf("filling buffer with 0xFF\n");
  memset(rx_buffer, 0xFF, sizeof(rx_buffer));
  new_buffer();

  // ===== ONBOARD LED SETUP =====
  printf("setup gpio\n");
  gpio_init(PIN_DEBUG);
  gpio_set_dir(PIN_DEBUG, GPIO_OUT);




  // ===== PIO SETUP =====
  printf("setup pio\n");

  PIO pio = pio0;
  uint sm = 0;
  uint offset = pio_add_program(pio, &pinstream_program);
  pio_sm_config c = pinstream_program_get_default_config(offset);

  // Map OUT to GPIO pins
  sm_config_set_out_pins(&c, PIN_BASE, PIN_COUNT);
  pio_sm_set_consecutive_pindirs(pio, sm, PIN_BASE, PIN_COUNT, true);

  sm_config_set_out_shift(&c, false, true, 8);
  sm_config_set_fifo_join(&c, PIO_FIFO_JOIN_TX);

  // Set clock divider (adjust speed here)
  float div = 12.00f;
  sm_config_set_clkdiv(&c, div);

  // Init GPIO function to PIO
  for (int i = PIN_BASE; i < PIN_BASE + PIN_COUNT; i++) {
    pio_gpio_init(pio, i);
    gpio_set_function(i, GPIO_FUNC_PIO0);
  }
  
  pio_sm_clear_fifos(pio, sm);
  pio_sm_restart(pio, sm);
  pio_sm_init(pio, sm, offset, &c);
  pio_sm_set_enabled(pio, sm, true);



  // ===== DMA SETUP =====
  printf("setup dma\n");

  dma_chan_data = dma_claim_unused_channel(true);

  dma_channel_config dma_chan_config_data =
      dma_channel_get_default_config(dma_chan_data);

  channel_config_set_transfer_data_size(&dma_chan_config_data, DMA_SIZE_8);
  channel_config_set_read_increment(&dma_chan_config_data, true);
  channel_config_set_write_increment(&dma_chan_config_data, false);
  channel_config_set_dreq(&dma_chan_config_data, pio_get_dreq(pio, sm, true));
  dma_channel_configure(dma_chan_data, &dma_chan_config_data, &pio->txf[sm],
                        *read_wave, actual_buffer_len, false);

  dma_channel_set_irq0_enabled(dma_chan_data, true);
  irq_set_exclusive_handler(DMA_IRQ_0, dma_irq_handler);
  irq_set_enabled(DMA_IRQ_0, true);
  dma_channel_start(dma_chan_data);




  // ===== I2C setup =====
  printf("setup i2c\n");

  i2c_init(i2c1, 100000);

  gpio_set_function(PIN_SDA, GPIO_FUNC_I2C);
  gpio_set_function(PIN_SCL, GPIO_FUNC_I2C);

  gpio_pull_up(PIN_SDA);
  gpio_pull_up(PIN_SCL);

  i2c_set_slave_mode(i2c1, true, 0x30);

  // Enable the interrupts we care about
  (void)i2c_get_hw(i2c1)->clr_intr;
  i2c_get_hw(i2c1)->intr_mask =
      I2C_IC_INTR_MASK_M_RX_FULL_BITS | I2C_IC_INTR_MASK_M_STOP_DET_BITS;

  irq_set_exclusive_handler(I2C1_IRQ, i2c1_irq_handler);
  irq_set_enabled(I2C1_IRQ, true);





  printf("beginning loop\n");
  //printf("write_wave:\n");
  //for (int i = 0; i < actual_buffer_len; i++) {
  //  printf("%d\n", write_wave[i]);
  //}
  while (true) {
    if (wave_dirty && rx_index == 0) {
        //flip_led();
        new_buffer();
        if (animation_done()) {
            wave_dirty = false;
        }
    }
    sleep_ms(10);
  }
}

int main() {
    stdio_init_all();

    sleep_ms(2000);

    printf("boot\n");

    init();
}
