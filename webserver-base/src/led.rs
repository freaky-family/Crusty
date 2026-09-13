use core::sync::atomic::{AtomicBool, Ordering};

use defmt::println;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Output;

pub static LED_STATE: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
pub async fn led_task(mut led: Output<'static>) {
    loop {
        if LED_STATE.load(Ordering::Relaxed) {
            led.set_high();
            println!("CACA");
        } else {
            led.set_low();
            println!("PIPI");
        }
        Timer::after(Duration::from_millis(50)).await;
    }
}
