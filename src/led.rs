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

pub struct Servos {
    pub servo1: esp_hal::peripherals::GPIO2<'static>, // Left front leg
    pub servo2: esp_hal::peripherals::GPIO4<'static>, // Left front leg
    pub servo3: esp_hal::peripherals::GPIO18<'static>, // Left back leg
    pub servo4: esp_hal::peripherals::GPIO19<'static>, // Left back leg
    pub servo5: esp_hal::peripherals::GPIO12<'static>, // Right front leg
    pub servo6: esp_hal::peripherals::GPIO14<'static>, // Right front leg
    pub servo7: esp_hal::peripherals::GPIO33<'static>, // Right back leg
    pub servo8: esp_hal::peripherals::GPIO32<'static>, // Right back leg
}

// struct Leg {
//     pub channel0: channel::Channel<'_, HighSpeed>,
//     pub channel1: channel::Channel<'_, HighSpeed>,
// }

enum LegNb {
    LF,
    RF,
    LB,
    RB
}

struct DutyInfo {
    pub max_duty_cycle: u32,
    pub max_duty: u32,
    pub min_duty: u32,
    pub duty_gap: u32,
    pub duty: u16
}

#[embassy_executor::task]
pub async fn led_task(mut servos: Servos, ledc: Ledc<'static>) {
    let mut hstimer0: timer::Timer<'_, HighSpeed> = ledc.timer::<HighSpeed>(timer::Number::Timer0);
    hstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty12Bit,
            clock_source: timer::HSClockSource::APBClk,
            frequency: Rate::from_hz(50),
        })
        .unwrap();

    let mut channel0: channel::Channel<'_, HighSpeed> = ledc.channel(channel::Number::Channel0, servos.servo1);
    channel0
    .configure(channel::config::Config {
        timer: &hstimer0,
        duty_pct: 10,
        drive_mode: DriveMode::PushPull,
    })
    .unwrap();

    let mut channel1 = ledc.channel(channel::Number::Channel1, servos.servo2);

    channel1.configure(channel::config::Config {
        timer: &hstimer0,
        duty_pct: 10,
        drive_mode: DriveMode::PushPull
    }).unwrap();


    // let mut leg1: Leg = Leg {channel0: &channel0, channel1: &channel1};
    let mut channel2: channel::Channel<'_, HighSpeed> = ledc.channel(channel::Number::Channel2, servos.servo5.reborrow());
    channel2
        .configure(channel::config::Config {
            timer: &hstimer0,
            duty_pct: 10,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    let mut channel3 = ledc.channel(channel::Number::Channel3, servos.servo6.reborrow());
    channel3.configure(channel::config::Config {
        timer: &hstimer0,
        duty_pct: 10,
        drive_mode: DriveMode::PushPull
    }).unwrap();

    let delay: Delay = Delay::new();

    let max_duty_cycle = channel0.max_duty_cycle() as u32;

    // Minimum duty (2.5%)
    // For 12bit -> 25 * 4096 /1000 => ~ 102
    let min_duty = (25 * max_duty_cycle) / 1000;
    // Maximum duty (12.5%)
    // For 12bit -> 125 * 4096 /1000 => 512
    let max_duty = (125 * max_duty_cycle) / 1000;
    // 512 - 102 => 410
    let duty_gap = max_duty - min_duty;

    let mut duty_info = DutyInfo {
        max_duty_cycle: max_duty_cycle, max_duty: max_duty, min_duty:min_duty, duty_gap: duty_gap, duty: 0
    };
    let mut is_high: u8;
    let mut old_state: u8 = 1;
    let mut state: u8;
    loop {
        state = LED_STATE.load(Ordering::Relaxed);
        if state == 0 {
            is_high = 0;
            if is_high != old_state {
                old_state = is_high;
                stop(& mut channel0, & mut channel1, & mut channel2, & mut channel3, & mut duty_info);
                println!("STOP");
            }
        } else if state == 1 {
            is_high = 1;
            if is_high != old_state {
                old_state = is_high;
                println!("WALK");
                stop(&mut channel0, &mut channel1, &mut channel2, &mut channel3, & mut duty_info);
            }
            walk(&mut channel0, &mut channel1, &mut channel2, &mut channel3, &mut duty_info, &delay);
        } else if state == 2 {
            is_high = 2;
            if is_high != old_state {
                println!("HELLO");
                old_state = is_high;
                stop(&mut channel0, &mut channel1, &mut channel2, &mut channel3, & mut duty_info);
            }
            hello(&mut channel0, &mut channel1, & mut duty_info, &delay);
        }
        Timer::after(Duration::from_millis(50)).await;
    }
}

fn duty_from_angle(deg: u32, min_duty: u32, duty_gap: u32) -> u16 {
    let duty = min_duty + ((deg * duty_gap) / 180);
    duty as u16
}

fn stop(channel0: & mut channel::Channel<'_, HighSpeed>, channel1:& mut channel::Channel<'_, HighSpeed>,
    channel2:& mut channel::Channel<'_, HighSpeed>, channel3:& mut channel::Channel<'_, HighSpeed>, duty_info: & mut DutyInfo) {
    duty_info.duty = duty_from_angle(0, duty_info.min_duty, duty_info.duty_gap);
    channel0.set_duty_cycle(duty_info.duty).unwrap();
    duty_info.duty = duty_from_angle(180, duty_info.min_duty, duty_info.duty_gap);
    channel2.set_duty_cycle(duty_info.duty).unwrap();
    duty_info.duty = duty_from_angle(90, duty_info.min_duty, duty_info.duty_gap);
    channel1.set_duty_cycle(duty_info.duty).unwrap();
    channel3.set_duty_cycle(duty_info.duty).unwrap();
}

fn hello(channel0: & mut channel::Channel<'_, HighSpeed>, channel1:& mut channel::Channel<'_, HighSpeed>,
    duty_info: & mut DutyInfo, delay: &Delay) {
    duty_info.duty = duty_from_angle(10, duty_info.min_duty, duty_info.duty_gap);
    channel1.set_duty_cycle(duty_info.duty).unwrap();
    delay.delay_millis(200);
    duty_info.duty = duty_from_angle(140, duty_info.min_duty, duty_info.duty_gap);
    channel0.set_duty_cycle(duty_info.duty).unwrap();
    delay.delay_millis(200);
    duty_info.duty = duty_from_angle(175, duty_info.min_duty, duty_info.duty_gap);
    channel0.set_duty_cycle(duty_info.duty).unwrap();
}

fn walk(channel0: & mut channel::Channel<'_, HighSpeed>, channel1:& mut channel::Channel<'_, HighSpeed>,
    channel2:& mut channel::Channel<'_, HighSpeed>, channel3:& mut channel::Channel<'_, HighSpeed>,
    duty_info: & mut DutyInfo, delay: &Delay) {

    // Step 1
    duty_info.duty = duty_from_angle(10, duty_info.min_duty, duty_info.duty_gap);
    channel0.set_duty_cycle(duty_info.duty).unwrap();
    channel1.set_duty_cycle(duty_info.duty).unwrap();
    duty_info.duty = duty_from_angle(50, duty_info.min_duty, duty_info.duty_gap);
    channel3.set_duty_cycle(duty_info.duty).unwrap();
    duty_info.duty = duty_from_angle(80, duty_info.min_duty, duty_info.duty_gap);
    channel2.set_duty_cycle(duty_info.duty).unwrap();
    delay.delay_millis(200);

    // Step 2
    duty_info.duty = duty_from_angle(120, duty_info.min_duty, duty_info.duty_gap);
    channel1.set_duty_cycle(duty_info.duty).unwrap();
    duty_info.duty = duty_from_angle(170, duty_info.min_duty, duty_info.duty_gap);
    channel3.set_duty_cycle(duty_info.duty).unwrap();
    delay.delay_millis(200);

    // Step 3
    duty_info.duty = duty_from_angle(100, duty_info.min_duty, duty_info.duty_gap);
    channel0.set_duty_cycle(duty_info.duty).unwrap();
    duty_info.duty = duty_from_angle(180, duty_info.min_duty, duty_info.duty_gap);
    channel2.set_duty_cycle(duty_info.duty).unwrap();
    delay.delay_millis(200);

    // Step 4
    duty_info.duty = duty_from_angle(0, duty_info.min_duty, duty_info.duty_gap);
    channel1.set_duty_cycle(duty_info.duty).unwrap();
    duty_info.duty = duty_from_angle(80, duty_info.min_duty, duty_info.duty_gap);
    channel3.set_duty_cycle(duty_info.duty).unwrap();
    delay.delay_millis(200);
}
