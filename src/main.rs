// rtic demo - April 3, 2022
// uart on pin gpio0, gpio1 (need usb to uart)
//     uart ttl -> https://www.adafruit.com/product/954
// led on pin gpio25
// ws2812 pin on gpio10
//
// this is my rust tutorial for myself
//     separate file for setup
//     uses pio on rtic
//     uses interrupt on uart
//
// five files in this tutorial to start rust project
//     1. main.rs - should be in the src directory
//     2. setup.rs - should be in the src directory
//     3. Cargo.toml - should be in the root project directory
//     4. memory.x - should be in the root project directory
//     5. config.toml - should be in the .cargo directory   

#![no_std]
#![no_main]

use panic_halt as _;
mod setup;

#[rtic::app(device = rp_pico::hal::pac, dispatchers = [XIP_IRQ])]
mod app {
   
    use crate::setup::setup;
    use crate::setup::LedPin;
    use crate::setup::UartType;
    use crate::setup::PioTx;
    use embedded_hal::digital::v2::ToggleableOutputPin;
    use rp2040_monotonic::*;
    
    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type Monotonic = Rp2040Monotonic;

    #[shared]
    struct Shared {uart: UartType, pio_tx: PioTx}

    #[local]
    struct Local {led: LedPin}

    #[init(local = [])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let (mono, led, uart, pio_tx) = setup(cx.device, cx.core);

        led_blinker::spawn().ok();

        (
            Shared {uart, pio_tx},
            Local {led},
            init::Monotonics(mono),
        )
    }

    // toggle led - shows we're stil alive
    #[task(local = [led])]
    fn led_blinker(cx: led_blinker::Context) {
        cx.local.led.toggle().ok();
        led_blinker::spawn_after(500.millis()).ok();
    }
    
    // get a char on serial port and change the w2812 color
    // base on the choice
    #[task(binds = UART0_IRQ, priority = 2, shared = [uart, pio_tx])]
    fn on_rx(cx: on_rx::Context){
      let mut data = [0u8; 1];
      let uart = cx.shared.uart;
      let pio_tx = cx.shared.pio_tx;

      (uart, pio_tx).lock(|uart_a, pio_tx_a|{
        match uart_a.read_full_blocking(&mut data){
          Err(_e) => {}
          Ok(_count) => {
            match data[0] { // change color of ws2812
              b'g' | b'G' => {pio_tx_a.write(0x000080);}, // green
              b'r' | b'R' => {pio_tx_a.write(0x008000);}, // red
              b'b' | b'B' => {pio_tx_a.write(0x080000);}, // blue
              _ =>           {pio_tx_a.write(0x000000);}, // off
              
            }
          }
        }
      });
    }
}
