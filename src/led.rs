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

struct DutyInfo {
    pub min_duty: u32,
    pub duty_gap: u32,
    pub duty: u16
}

const ID1: usize = 0;
const ID2: usize = 1;
const ID3: usize = 2;
const ID4: usize = 3;

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

    // LF Leg
    let mut cl: [channel::Channel<'_, HighSpeed>; 4] = [ledc.channel(channel::Number::Channel1, servos.servo2.reborrow()), ledc.channel(channel::Number::Channel5, servos.servo4.reborrow()),
        ledc.channel(channel::Number::Channel0, servos.servo1.reborrow()), ledc.channel(channel::Number::Channel4, servos.servo3.reborrow())];
    let mut cr: [channel::Channel<'_, HighSpeed>; 4] = [ledc.channel(channel::Number::Channel3, servos.servo6.reborrow()), ledc.channel(channel::Number::Channel7, servos.servo8.reborrow()),
    ledc.channel(channel::Number::Channel2, servos.servo5.reborrow()), ledc.channel(channel::Number::Channel6, servos.servo7.reborrow())];
    // cl[ID3] = ledc.channel(channel::Number::Channel0, servos.servo1.reborrow());
    for i in 0..4 {
        cl[i].configure(channel::config::Config {
            timer: &hstimer0,
            duty_pct: 10,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
        cr[i].configure(channel::config::Config {
            timer: &hstimer0,
            duty_pct: 10,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();
    }

    let delay: Delay = Delay::new();

    let max_duty_cycle = cl[ID3].max_duty_cycle() as u32;

    // Minimum duty (2.5%)
    // For 12bit -> 25 * 4096 /1000 => ~ 102
    let min_duty = (25 * max_duty_cycle) / 1000;
    // Maximum duty (12.5%)
    // For 12bit -> 125 * 4096 /1000 => 512
    let max_duty = (125 * max_duty_cycle) / 1000;
    // 512 - 102 => 410
    let duty_gap = max_duty - min_duty;

    let mut duty_info = DutyInfo {
        min_duty:min_duty, duty_gap: duty_gap, duty: 0
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
                println!("STOP");
                stop(& mut cl, & mut cr, & mut duty_info);
            }
        } else if state == 1 {
            is_high = 1;
            if is_high != old_state {
                old_state = is_high;
                println!("WALK");
                stop(& mut cl, & mut cr, & mut duty_info);
            }
            walk(& mut cl, & mut cr, &mut duty_info, &delay);
        } else if state == 2 {
            is_high = 2;
            if is_high != old_state {
                println!("HELLO");
                old_state = is_high;
                stop(& mut cl, & mut cr, & mut duty_info);
            }
            hello(& mut cl, & mut cr, & mut duty_info, &delay);
        } else if state == 3 {
            is_high = 3;
            if is_high != old_state {
                println!("TURN RIGHT");
                old_state = is_high;
                stop(& mut cl, & mut cr, & mut duty_info);
            }
            turn_right(& mut cl, & mut cr, &mut duty_info, &delay);
        } else if state == 4 {
            is_high = 4;
            if is_high != old_state {
                println!("TURN RIGHT");
                old_state = is_high;
                stop(& mut cl, & mut cr, & mut duty_info);
            }
            turn_left(& mut cl, & mut cr, &mut duty_info, &delay);
        } else if state == 5 {
            is_high = 4;
            if is_high != old_state {
                println!("freaky");
                old_state = is_high;
                stop(& mut cl, & mut cr, & mut duty_info);
            }
            freaky(& mut cl, & mut cr, &mut duty_info, &delay);
        }
        Timer::after(Duration::from_millis(50)).await;
    }
}

fn duty_from_angle(deg: u32, min_duty: u32, duty_gap: u32) -> u16 {
    let duty = min_duty + ((deg * duty_gap) / 180);
    duty as u16
}

fn stop(cl: & mut [channel::Channel<'_, HighSpeed>; 4],
    cr: & mut [channel::Channel<'_, HighSpeed>; 4],
    duty_info: & mut DutyInfo) {
    set_duty_on_channel(& mut cl[ID3], 0, duty_info);
    set_duty_on_channel(& mut cr[ID4], 0, duty_info);

    set_duty_on_channel(& mut cr[ID3], 180, duty_info);
    set_duty_on_channel(& mut cl[ID4], 180, duty_info);
    set_duty_on_channel(& mut cl[ID1], 90, duty_info);
    set_duty_on_channel(& mut cr[ID1], 90, duty_info);
    set_duty_on_channel(& mut cl[ID2], 90, duty_info);
    set_duty_on_channel(& mut cr[ID2], 90, duty_info);
}

fn hello(cl: & mut [channel::Channel<'_, HighSpeed>; 4],
    cr: & mut [channel::Channel<'_, HighSpeed>; 4],
    duty_info: & mut DutyInfo, delay: &Delay) {
    set_duty_on_channel(& mut cr[ID1], 170, duty_info);
    set_duty_on_channel(& mut cl[ID1], 10, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cl[ID3], 140, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cl[ID3], 175, duty_info);
}

fn walk(cl: & mut [channel::Channel<'_, HighSpeed>; 4],
    cr: & mut [channel::Channel<'_, HighSpeed>; 4],
    duty_info: & mut DutyInfo, delay: &Delay) {

    set_duty_on_channel(& mut cr[ID3], 135, duty_info);
    set_duty_on_channel(& mut cl[ID3], 0, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cl[ID4], 135, duty_info);
    set_duty_on_channel(& mut cl[ID2], 90, duty_info);
    set_duty_on_channel(& mut cr[ID4], 0, duty_info);
    set_duty_on_channel(& mut cr[ID1], 180, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cr[ID2], 45, duty_info);
    set_duty_on_channel(& mut cl[ID1], 90, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cr[ID4], 45, duty_info);
    set_duty_on_channel(& mut cl[ID4], 180, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cr[ID3], 180, duty_info);
    set_duty_on_channel(& mut cl[ID3], 45, duty_info);
    set_duty_on_channel(& mut cr[ID2], 90, duty_info);
    set_duty_on_channel(& mut cl[ID1], 0, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cl[ID2], 135, duty_info);
    set_duty_on_channel(& mut cr[ID1], 90, duty_info);
    delay.delay_millis(200);

}

fn turn_right(cl: & mut [channel::Channel<'_, HighSpeed>; 4],
    cr: & mut [channel::Channel<'_, HighSpeed>; 4],
    duty_info: & mut DutyInfo, delay: &Delay) {

    set_duty_on_channel(& mut cr[ID3], 180, duty_info);
    set_duty_on_channel(& mut cl[ID3], 45, duty_info);
    set_duty_on_channel(& mut cl[ID4], 180, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cr[ID1], 180, duty_info);
    set_duty_on_channel(& mut cl[ID1], 0, duty_info);
    set_duty_on_channel(& mut cl[ID2], 135, duty_info);
    set_duty_on_channel(& mut cr[ID4], 45, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cl[ID3], 0, duty_info);
    set_duty_on_channel(& mut cr[ID3], 135, duty_info);
    set_duty_on_channel(& mut cr[ID2], 45, duty_info);
    set_duty_on_channel(& mut cr[ID4], 0, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cl[ID4], 45, duty_info);
    set_duty_on_channel(& mut cr[ID2], 90, duty_info);
    set_duty_on_channel(& mut cl[ID1], 90, duty_info);
    set_duty_on_channel(& mut cr[ID1], 90, duty_info);
    set_duty_on_channel(& mut cl[ID2], 90, duty_info);
    delay.delay_millis(200);
}

fn turn_left(cl: & mut [channel::Channel<'_, HighSpeed>; 4],
    cr: & mut [channel::Channel<'_, HighSpeed>; 4],
    duty_info: & mut DutyInfo, delay: &Delay) {

    set_duty_on_channel(& mut cl[ID3], 0, duty_info);
    set_duty_on_channel(& mut cr[ID3], 135, duty_info);
    set_duty_on_channel(& mut cr[ID4], 0, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cl[ID1], 0, duty_info);
    set_duty_on_channel(& mut cr[ID1], 180, duty_info);
    set_duty_on_channel(& mut cr[ID2], 45, duty_info);
    set_duty_on_channel(& mut cl[ID4], 135, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cr[ID3], 180, duty_info);
    set_duty_on_channel(& mut cl[ID3], 45, duty_info);
    set_duty_on_channel(& mut cl[ID2], 135, duty_info);
    set_duty_on_channel(& mut cl[ID4], 180, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cr[ID4], 135, duty_info);
    set_duty_on_channel(& mut cl[ID2], 90, duty_info);
    set_duty_on_channel(& mut cr[ID1], 90, duty_info);
    set_duty_on_channel(& mut cl[ID1], 90, duty_info);
    set_duty_on_channel(& mut cr[ID2], 90, duty_info);
    delay.delay_millis(200);
}

fn freaky(cl: & mut [channel::Channel<'_, HighSpeed>; 4],
    cr: & mut [channel::Channel<'_, HighSpeed>; 4],
    duty_info: & mut DutyInfo, delay: &Delay) {

    set_duty_on_channel(& mut cr[ID1], 180, duty_info);
    set_duty_on_channel(& mut cl[ID1], 0, duty_info);
    set_duty_on_channel(& mut cr[ID3], 120, duty_info);
    set_duty_on_channel(& mut cl[ID3], 60, duty_info);
    set_duty_on_channel(& mut cl[ID2], 90, duty_info);
    set_duty_on_channel(& mut cr[ID2], 90, duty_info);
    set_duty_on_channel(& mut cl[ID4], 180, duty_info);
    set_duty_on_channel(& mut cr[ID4], 0, duty_info);
    delay.delay_millis(200);
    set_duty_on_channel(& mut cl[ID4], 135, duty_info);
    set_duty_on_channel(& mut cr[ID4], 45, duty_info);
    delay.delay_millis(200);
}


fn set_duty_on_channel(channel: & mut channel::Channel<'_, HighSpeed>, deg: u32, duty_info: & mut DutyInfo) {
    duty_info.duty = duty_from_angle(deg, duty_info.min_duty, duty_info.duty_gap);
    channel.set_duty_cycle(duty_info.duty).unwrap();
}
