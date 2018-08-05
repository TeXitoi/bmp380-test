#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;
extern crate cortex_m_semihosting as sh;
extern crate bme280;

use rt::ExceptionFrame;
use hal::prelude::*;
use core::fmt::Write;

entry!(main);

fn main() -> ! {
    let dp = hal::stm32f103xx::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let delay = hal::delay::Delay::new(cp.SYST, clocks);

    let mut hstdout = sh::hio::hstdout().unwrap();

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let pb6 = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let pb7 = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);
    let i2c = hal::i2c::I2c::i2c1(
        dp.I2C1,
        (pb6, pb7),
        &mut afio.mapr,
        hal::i2c::Mode::Fast {
            frequency: 400_000,
            duty_cycle: hal::i2c::DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
    );
    let i2c = hal::i2c::blocking_i2c(i2c, clocks, 100, 100, 100, 100);
    let mut bme280 = bme280::BME280::new_primary(i2c, delay);
    bme280.init().unwrap();

    loop {
        let measurements = bme280.measure().unwrap();
        writeln!(hstdout, "Relative Humidity = {}%", measurements.humidity).unwrap();
        writeln!(hstdout, "Temperature = {} deg C", measurements.temperature).unwrap();
        writeln!(hstdout, "Pressure = {} pascals", measurements.pressure).unwrap();
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
