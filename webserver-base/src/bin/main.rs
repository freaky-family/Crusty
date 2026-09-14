#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

// use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::ledc::{Ledc};
use webserver_html as lib;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 98767);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");
    let radio_init = &*lib::mk_static!(
        esp_radio::Controller<'static>,
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller")
    );
    let rng = Rng::new();

    let servo: esp_hal::peripherals::GPIO2<'_> = peripherals.GPIO2;
    let ledc: Ledc<'_> = Ledc::new(peripherals.LEDC);
    // let i2c_bus: I2c<'_, esp_hal::Async> = I2c::new(
    //     peripherals.I2C0,
    //     // I2cConfig is alias of esp_hal::i2c::master::I2c::Config
    //     I2cConfig::default().with_frequency(Rate::from_khz(400)),
    // )
    // .unwrap()
    // .with_scl(peripherals.GPIO18)
    // .with_sda(peripherals.GPIO23)
    // .into_async();
    let stack = lib::wifi::start_wifi(radio_init, peripherals.WIFI, rng, &spawner).await;
    spawner.must_spawn(lib::led::led_task(servo, ledc));

    // let interface = I2CDisplayInterface::new(i2c_bus);
    // // // initialize the display
    // let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
    //     .into_buffered_graphics_mode();
    // display.init().await.unwrap();

    // let text_style = MonoTextStyleBuilder::new()
    //     .font(&FONT_6X10)
    //     .text_color(BinaryColor::On)
    //     .build();

    // Text::with_baseline("Hello, Rust!", Point::new(0, 16), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();
    // display.flush().await.unwrap();

    let web_app = lib::web::WebApp::default();
    for id in 0..lib::web::WEB_TASK_POOL_SIZE {
        spawner.must_spawn(lib::web::web_task(
            id,
            stack,
            web_app.router,
            web_app.config,
        ));
    }

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
