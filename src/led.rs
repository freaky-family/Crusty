use core::sync::atomic::{AtomicU8, Ordering};

use defmt::println;
use embassy_time::{Duration, Timer};
use esp_hal::ledc::{HighSpeed, Ledc, channel, timer};
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::time::Rate;
use esp_hal::gpio::DriveMode;
use esp_hal::ledc::timer::TimerIFace;
use embedded_hal::pwm::SetDutyCycle;
// use led::mono::LED;
use esp_hal::delay::Delay;

pub static LED_STATE: AtomicU8 = AtomicU8::new(0);

#[embassy_executor::task]
pub async fn led_task(mut servo: esp_hal::peripherals::GPIO2<'static>, mut servo2: esp_hal::peripherals::GPIO4<'static>, ledc: Ledc<'static>) {
    let mut hstimer0 = ledc.timer::<HighSpeed>(timer::Number::Timer0);
    hstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty12Bit,
            clock_source: timer::HSClockSource::APBClk,
            frequency: Rate::from_hz(50),
        })
        .unwrap();

    let mut channel0 = ledc.channel(channel::Number::Channel0, servo.reborrow());
    channel0
        .configure(channel::config::Config {
            timer: &hstimer0,
            duty_pct: 10,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    let mut channel1 = ledc.channel(channel::Number::Channel1, servo2.reborrow());
    channel1.configure(channel::config::Config {
        timer: &hstimer0,
        duty_pct: 10,
        drive_mode: DriveMode::PushPull
    }).unwrap();
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

    let mut is_high: u8;
    let mut old_state: u8 = 1;
    let mut duty: u16;
    loop {
        let state: u8 = LED_STATE.load(Ordering::Relaxed);
        if state == 0 {
            is_high = 0;
            if is_high != old_state {
                old_state = is_high;
                duty = duty_from_angle(0, min_duty, duty_gap);
                channel0.set_duty_cycle(duty).unwrap();
                duty = duty_from_angle(90, min_duty, duty_gap);
                channel1.set_duty_cycle(duty).unwrap();
                println!("STOP");
            }
        } else if state == 1 {
            is_high = 1;
            if is_high != old_state {
                old_state = is_high;
                duty = duty_from_angle(5, min_duty, duty_gap);
                channel0.set_duty_cycle(duty).unwrap();
                channel1.set_duty_cycle(duty).unwrap();
                println!("WALK");
            }
        } else if state == 2 {
            is_high = 2;
            if is_high != old_state {
                println!("HELLO");
                old_state = is_high;
            }
            duty = duty_from_angle(90, min_duty, duty_gap);
            channel1.set_duty_cycle(duty).unwrap();
            delay.delay_millis(200);
            duty = duty_from_angle(140, min_duty, duty_gap);
            channel0.set_duty_cycle(duty).unwrap();
            delay.delay_millis(200);
            duty = duty_from_angle(175, min_duty, duty_gap);
            channel0.set_duty_cycle(duty).unwrap();
        }
        Timer::after(Duration::from_millis(50)).await;
    }
}

fn duty_from_angle(deg: u32, min_duty: u32, duty_gap: u32) -> u16 {
    let duty = min_duty + ((deg * duty_gap) / 180);
    duty as u16
}
