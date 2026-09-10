#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::fmt::Write;

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::{main, uart};

// LEDC
use esp_hal::gpio::DriveMode;
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{HighSpeed, Ledc, channel, timer};
use esp_hal::uart::Config;
use esp_hal::{uart::Uart};
use esp_hal::time::Rate;

use embedded_hal::pwm::SetDutyCycle;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 1.0.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut servo1 = peripherals.GPIO2;
    let mut servo2 = peripherals.GPIO4;
    let ledc = Ledc::new(peripherals.LEDC);
    // let ledc1 = Ledc::new(peripherals.LEDC);

    let mut hstimer0 = ledc.timer::<HighSpeed>(timer::Number::Timer0);
    hstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty12Bit,
            clock_source: timer::HSClockSource::APBClk,
            frequency: Rate::from_hz(50),
        })
        .unwrap();

    let mut channel0 = ledc.channel(channel::Number::Channel0, servo1.reborrow());
    let mut channel1 = ledc.channel(channel::Number::Channel1, servo2.reborrow());
    channel0
        .configure(channel::config::Config {
            timer: &hstimer0,
            duty_pct: 10,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    channel1
        .configure(channel::config::Config {
            timer: &hstimer0,
            duty_pct: 10,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    let delay = Delay::new();

    let max_duty_cycle = channel0.max_duty_cycle() as u32;

    // Minimum duty (2.5%)
    // For 12bit -> 25 * 4096 /1000 => ~ 102
    let min_duty = (25 * max_duty_cycle) / 1000;
    // Maximum duty (12.5%)
    // For 12bit -> 125 * 4096 /1000 => 512
    let max_duty = (125 * max_duty_cycle) / 1000;
    // 512 - 102 => 410
    let duty_gap = max_duty - min_duty;


    let mut angle:i32 = 90;

    let mut uart0 = Uart::new(peripherals.UART0, Config::default()).unwrap();
    let mut buffer: [u8; _] = [0; 50];
    let mut size = 0;
    loop {
        // println!("Type shit");
        // delay.delay_millis(500);

        // delay.delay_millis(500);
        // writeln!(uart0, "Hello world");

        let mut letter = [0; 1];
        match uart0.read(&mut letter) {
            Ok(_) => {
                writeln!(uart0, "{:?}", letter);
                if letter[0] == 115 {
                    // S
                    for deg in 0..=180 {
                        let duty = duty_from_angle(deg, min_duty, duty_gap);
                        channel0.set_duty_cycle(duty).unwrap();
                        // delay.delay_millis(10);
                    }
                } else if letter[0] == 122 {
                    // Z
                    for deg in (0..=180).rev() {
                        let duty = duty_from_angle(deg, min_duty, duty_gap);
                        channel0.set_duty_cycle(duty).unwrap();
                        // delay.delay_millis(10);
                    }
                } else if letter[0] == 113 {
                    // Q
                    for deg in (0..=180).rev() {
                        let duty = duty_from_angle(deg, min_duty, duty_gap);
                        channel1.set_duty_cycle(duty).unwrap();
                        // delay.delay_millis(10);
                    }
                } else if letter[0] == 100 {
                    // D
                    for deg in 0..=180 {
                        let duty = duty_from_angle(deg, min_duty, duty_gap);
                        channel1.set_duty_cycle(duty).unwrap();
                        // delay.delay_millis(10);
                    }
                }
            },
            Err(_) => {},
        };
    }
}

fn duty_from_angle(deg: u32, min_duty: u32, duty_gap: u32) -> u16 {
    let duty = min_duty + ((deg * duty_gap) / 180);
    duty as u16
}


// #![no_std]
// #![no_main]
// #![deny(
//     clippy::mem_forget,
//     reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
//     holding buffers for the duration of a data transfer."
// )]

// use esp_hal::clock::CpuClock;
// use esp_hal::delay::Delay;
// use esp_hal::main;

// // LEDC
// use esp_hal::gpio::DriveMode;
// use esp_hal::ledc::channel::ChannelIFace;
// use esp_hal::ledc::timer::TimerIFace;
// use esp_hal::ledc::{HighSpeed, Ledc, channel, timer};
// use esp_hal::time::Rate;

// use embedded_hal::pwm::SetDutyCycle;

// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

// // This creates a default app-descriptor required by the esp-idf bootloader.
// // For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
// esp_bootloader_esp_idf::esp_app_desc!();

// #[main]
// fn main() -> ! {
//     // generator version: 1.0.0

//     let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
//     let peripherals = esp_hal::init(config);

//     let mut servo = peripherals.GPIO33;
//     let ledc = Ledc::new(peripherals.LEDC);

//     let mut hstimer0 = ledc.timer::<HighSpeed>(timer::Number::Timer0);
//     hstimer0
//         .configure(timer::config::Config {
//             duty: timer::config::Duty::Duty12Bit,
//             clock_source: timer::HSClockSource::APBClk,
//             frequency: Rate::from_hz(50),
//         })
//         .unwrap();

//     let mut channel0 = ledc.channel(channel::Number::Channel0, servo.reborrow());
//     channel0
//         .configure(channel::config::Config {
//             timer: &hstimer0,
//             duty_pct: 10,
//             drive_mode: DriveMode::PushPull,
//         })
//         .unwrap();

//     let delay = Delay::new();

//     let max_duty_cycle = channel0.max_duty_cycle() as u32;

//     // Minimum duty (2.5%)
//     // For 12bit -> 25 * 4096 /1000 => ~ 102
//     let min_duty = (25 * max_duty_cycle) / 1000;
//     // Maximum duty (12.5%)
//     // For 12bit -> 125 * 4096 /1000 => 512
//     let max_duty = (125 * max_duty_cycle) / 1000;
//     // 512 - 102 => 410
//     let duty_gap = max_duty - min_duty;

//     loop {
//         for deg in 0..=180 {
//             let duty = duty_from_angle(deg, min_duty, duty_gap);
//             channel0.set_duty_cycle(duty).unwrap();
//             delay.delay_millis(10);
//         }
//         delay.delay_millis(500);

//         for deg in (0..=180).rev() {
//             let duty = duty_from_angle(deg, min_duty, duty_gap);
//             channel0.set_duty_cycle(duty).unwrap();
//             delay.delay_millis(10);
//         }
//         delay.delay_millis(1000);
//     }
// }

// fn duty_from_angle(deg: u32, min_duty: u32, duty_gap: u32) -> u16 {
//     let duty = min_duty + ((deg * duty_gap) / 180);
//     duty as u16
// }
