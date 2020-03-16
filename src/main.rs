#![no_main]
#![no_std]

// On dev builds, go to debugger on panic
#[cfg(debug_assertions)]
extern crate panic_semihosting;

// On release builds, halt on panic
#[cfg(not(debug_assertions))]
extern crate panic_halt;

use cortex_m::peripheral::DWT;
use f3::hal::gpio::GpioExt;
use f3::hal::rcc::RccExt;
use f3::hal::stm32f30x;
use f3::led::Leds;
use rtfm::cyccnt::{Instant, U32Ext, CYCCNT};
use rtfm::Monotonic;

// const LED_PERIOD: u32 = 2_000_000;
// const LED_ON_TIME: u32 = 1_000_000;

const LED_PERIOD_MILLIS: u32 = 1_000;
const LED_ON_TIME_MILLIS: u32 = 100;

const CYC_PER_MILLI: u32 = 36_00;

const LED_PERIOD_CYC: u32 = LED_PERIOD_MILLIS * CYC_PER_MILLI;
const LED_ON_TIME_CYC: u32 = LED_ON_TIME_MILLIS * CYC_PER_MILLI;

#[rtfm::app(device = stm32f30x, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        leds: Leds,
    }

    /// Initialization section
    ///
    /// Initializes our tasks and peripherals.
    #[init(schedule = [blink_led, turn_off_led])]
    fn init(mut cx: init::Context) -> init::LateResources {
        // Initialize (enable) the monotonic timer (CYCCNT)
        cx.core.DCB.enable_trace();

        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        // grab current time
        let now = Instant::now();

        // schedule turn on led to happen immediately after init
        cx.schedule.blink_led(now).unwrap();

        // schedule led turn off to occur after our duty period
        cx.schedule.turn_off_led(now + LED_ON_TIME_CYC.cycles()).unwrap();

        // get peripherals
        let board_peripherals = stm32f30x::Peripherals::take().unwrap();

        // acquire the RCC
        let mut rcc = board_peripherals.RCC.constrain();

        // acquire GPIOE, which operates the LEDs on this board
        let led_gpio = board_peripherals.GPIOE.split(&mut rcc.ahb);

        // create an LEDs object from the GPIO
        let leds = f3::led::Leds::new(led_gpio);

        // return resources
        init::LateResources {
            leds
        }
    }

    /// Turns the LED on.
    ///
    /// This task reschedules itself, and then turns on the LEDs.
    #[task(schedule = [blink_led], resources = [leds])]
    fn blink_led(cx: blink_led::Context) {
        // reschedule this task
        // this is done with the scheduled time rather than current clock time, preventing drift.
        cx.schedule.blink_led(cx.scheduled + LED_PERIOD_CYC.cycles()).unwrap();

        // grab leds
        let leds: &mut Leds = cx.resources.leds;

        // turn on leds
        set_leds(leds, true);
    }

    /// Turns the LED off.
    ///
    /// This task reschedules itself, and then turns off the LEDs.
    #[task(schedule = [turn_off_led], resources = [leds])]
    fn turn_off_led(cx: turn_off_led::Context) {
        // reschedule this task
        // this is done with the scheduled time rather than current clock time, preventing drift.
        cx.schedule.turn_off_led(cx.scheduled + LED_PERIOD_CYC.cycles()).unwrap();

        // grab leds
        let leds: &mut Leds = cx.resources.leds;

        // turn off leds
        set_leds(leds, false);
    }

    /// Declare free interrupts
    /// These are used to dispatch software-spawned tasks
    /// These are interrupts not used by hardware.
    extern "C" {
        fn CAN_RX1();
        fn CAN_RX2();
    }
};

fn set_leds(leds: &mut Leds, val: bool) {
    for led in leds.iter_mut() {
        set_led(led, val)
    }
}

fn set_led(led: &mut f3::led::Led, val: bool) {
    if val {
        led.on();
    } else {
        led.off()
    }
}