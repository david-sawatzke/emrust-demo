#![no_std]
#![no_main]

#[allow(unused)]
use panic_halt;

mod tnarx;

use core::fmt::Write;
use embedded_graphics::fonts::Font12x16;
use embedded_graphics::prelude::*;
use hd44780_driver::{Cursor, CursorBlink, HD44780};
use ili9341;
use shift_register_driver::sipo::ShiftRegister16;
use ssd1306::prelude::*;
use ssd1306::Builder;
use stm32f0xx_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::i2c::I2c;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::hal::stm32;
use crate::hal::time::{Hertz, KiloHertz};
use crate::hal::timers::*;
use nb::block;

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

use core::fmt::Debug;
use core::iter;
use embedded_hal;
#[entry]
fn main() -> ! {
    let p = stm32::Peripherals::take().unwrap();
    let cp = Peripherals::take().unwrap();
    /* Constrain clocking registers */
    let rcc = p.RCC.constrain();
    /* Configure clock to 8 MHz (i.e. the default) and freeze it */
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpiof = p.GPIOF.split();

    /* Get delay provider */
    let mut delay = Delay::new(cp.SYST, clocks);
    let mut timer = Timer::tim1(p.TIM1, Hertz(3), clocks);

    // I think the lcd needs a bit to initialize
    for _ in 0..10 {
        block!(timer.wait()).ok();
    }

    let scl = gpioa
        .pa9
        .into_alternate_af4()
        .internal_pull_up(true)
        .set_open_drain();
    let sda = gpioa
        .pa10
        .into_alternate_af4()
        .internal_pull_up(true)
        .set_open_drain();
    let i2c = I2c::i2c1(p.I2C1, (scl, sda), KiloHertz(400));

    // Configure pins for SPI
    let sck = gpioa.pa5.into_alternate_af0();
    let miso = gpioa.pa6.into_alternate_af0();
    let mosi = gpioa.pa7.into_alternate_af0();

    // Configure SPI with 100kHz rate
    let spi = Spi::spi1(p.SPI1, (sck, miso, mosi), ili9341::MODE, 48.mhz(), clocks);

    let cs = gpioa.pa2.into_push_pull_output();
    let dc = gpioa.pa3.into_push_pull_output();
    let reset = gpioa.pa4.into_push_pull_output();

    let mut disp: GraphicsMode<_> = Builder::new()
        .with_size(DisplaySize::Display128x32)
        .connect_i2c(i2c)
        .into();
    disp.init().unwrap();
    disp.flush().unwrap();

    let mut disp_ili = ili9341::Ili9341::new(spi, cs, dc, reset, &mut delay).unwrap();
    disp_ili
        .set_orientation(ili9341::Orientation::LandscapeFlipped)
        .unwrap();
    clear(&mut disp_ili);

    let clock = gpiof.pf1.into_push_pull_output();
    let latch = gpiof.pf0.into_push_pull_output();
    let data = gpiob.pb1.into_push_pull_output();
    let shift_register = ShiftRegister16::new(clock, latch, data);
    let mut outputs = shift_register.decompose();
    let mut it = outputs.iter_mut();
    let _ = it.next().unwrap();
    let d4 = it.next().unwrap();
    let d5 = it.next().unwrap();
    let d6 = it.next().unwrap();
    let d7 = it.next().unwrap();
    let rs = it.next().unwrap();
    let en = it.next().unwrap();
    let _ = it.next().unwrap();
    // Shift 1 Done
    let _pcdrst = it.next().unwrap();
    // this pin is unused
    let _pcdled = it.next().unwrap();
    let _ = it.next().unwrap();
    let _pcddc = it.next().unwrap();
    let _pcdce = it.next().unwrap();
    let tnadi = it.next().unwrap();
    let tnack = it.next().unwrap();
    let tnace = it.next().unwrap();
    // Shift 2 done
    let mut disp_hd44780 = HD44780::new_4bit(rs, en, d4, d5, d6, d7, delay);
    disp_hd44780.set_cursor_visibility(Cursor::Invisible);
    disp_hd44780.set_cursor_blink(CursorBlink::Off);

    let _pcddin = gpioa.pa1.into_push_pull_output();
    let _pcdclk = gpioa.pa0.into_push_pull_output();
    let mut disp_tna = tnarx::Tnarx::new(tnace, tnack, tnadi);

    let text_ssd = [
        "Consistent APIs",
        "Under 16KiB",
        "Highly Optimized",
        "Cross-vendor APIs",
        "Sane depedencies",
        "Advanced features",
        "Non-vendor specific",
    ];
    let mut text_ssd_counter = 0;
    let text_ili = [
        "This is (mostly) based on existing libs",
        "Source available on https://github.com/david-sawatzke/emrust-demo",
        "Using embedded_hal, libraries can be easiliy used on all suported platforms",
        "Register access is the same across families and even vendors",
        "This uses embedded-hal, stm32f0xx-hal, svd2rust, embedded-graphics, shift-register-driver, the display drives, and (of course) rust",
    ];
    let mut text_ili_counter = 0;
    let text_hd = [
        "Prevents race",
        "conditions",
        "Little-to-no",
        "overhead",
        "Checked register",
        "access",
        "No proprietary",
        "code necessary",
    ];
    let mut text_hd_counter = 0;
    let text_tna = ["STABLE", "CHECKED", "SAFE", "FAST", "POWERFULL"];
    let mut text_tna_counter = 0;
    loop {
        disp_hd44780.reset();
        disp_hd44780.clear();
        disp_hd44780.write_str(text_hd[text_hd_counter]);
        // Move the cursor to the second line
        disp_hd44780.set_cursor_pos(40);
        disp_hd44780.write_str(text_hd[text_hd_counter + 1]);
        text_hd_counter += 2;
        if text_hd_counter == text_hd.len() {
            text_hd_counter = 0;
        }

        disp.clear();
        text(&mut disp, text_ssd[text_ssd_counter], 10, 2).unwrap();
        text_ssd_counter += 1;
        if text_ssd_counter == text_ssd.len() {
            text_ssd_counter = 0;
        }
        disp.flush().unwrap();

        clear(&mut disp_ili);
        text(&mut disp_ili, text_ili[text_ili_counter], 26, 10).unwrap();
        text_ili_counter += 1;
        if text_ili_counter == text_ili.len() {
            text_ili_counter = 0;
        }
        disp_tna.erase();
        disp_tna.write_str(text_tna[text_tna_counter]);
        text_tna_counter += 1;
        if text_tna_counter == text_tna.len() {
            text_tna_counter = 0;
        }
        disp_tna.flush();

        // We (probably) have an overflow with 1Hz
        for _ in 0..10 {
            block!(timer.wait()).ok();
        }
    }
}

fn text<DISP, C>(disp: &mut DISP, text: &str, x: u8, y: u8) -> Result<(), ()>
where
    DISP: Drawing<C>,
    C: embedded_graphics::pixelcolor::PixelColor,
{
    if text.len() > x as usize * y as usize {
        return Err(());
    }
    let mut pos = 0;
    let mut y_pos = 0;
    while pos < text.len() {
        let remaining = if x as usize <= text.len() - 1 - pos {
            x as usize
        } else {
            text.len() - pos
        };
        disp.draw(
            Font12x16::render_str(&text[pos..pos + remaining])
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(0, 16 * y_pos))
                .into_iter(),
        );
        pos += remaining;
        y_pos += 1;
    }
    Ok(())
}

fn clear<E, SPI, CS, DC, RESET>(disp: &mut ili9341::Ili9341<SPI, CS, DC, RESET>)
where
    SPI: embedded_hal::blocking::spi::Transfer<u8, Error = E>
        + embedded_hal::blocking::spi::Write<u8, Error = E>,
    SPI: embedded_hal::blocking::spi::Transfer<u8>,
    SPI: embedded_hal::blocking::spi::Write<u8>,
    CS: embedded_hal::digital::OutputPin,
    DC: embedded_hal::digital::OutputPin,
    RESET: embedded_hal::digital::OutputPin,
    E: Debug,
{
    let iterate = iter::repeat(0xFFFF).take(320 * 240);
    disp.draw_iter(0, 0, 320, 240, iterate);
}
