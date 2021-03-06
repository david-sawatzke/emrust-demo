// TODO We leave all delays out, as we run on a shift register
// Adapted from https://github.com/niccokunzmann/UC121902-TNARX-A/
use embedded_hal::digital::OutputPin;

static LOOKUP_TABLE: &'static [u8] = &[
    //0b0000000, /*  ' '   */
    //0b0010010, /*  '!'   */
    //0b0110000, /*  '"'   */
    //0b1011100, /*  '#'   */
    //0b1011100, /*  '$'   */
    //0b1001001, /*  '%'   */
    //0b1011100, /*  '&'   */
    //0b0100000, /*  "'"   */
    //0b1100101, /*  '('   */
    //0b1010011, /*  ')'   */
    //0b1011100, /*  '*'   */
    //0b1011100, /*  '+'   */
    //0b0000010, /*  ','   */
    //0b0001000, /*  '-'   */
    //0b0000001, /*  '.'   */
    //0b0011100, /*  '/'   */
    //0b1110111, /*  '0'   */
    //0b0010010, /*  '1'   */
    //0b1011101, /*  '2'   */
    //0b1011011, /*  '3'   */
    //0b0111010, /*  '4'   */
    //0b1101011, /*  '5'   */
    //0b1101111, /*  '6'   */
    //0b1010010, /*  '7'   */
    //0b1111111, /*  '8'   */
    //0b1111011, /*  '9'   */
    //0b0001001, /*  ':'   */
    //0b0001010, /*  ';'   */
    //0b1101000, /*  '<'   */
    //0b0001001, /*  '='   */
    //0b1011000, /*  '>'   */
    //0b1011100, /*  '?'   */
    //0b1011100, /*  '@'   */
    0b1111110, /*  'A'   */
    0b0101111, /*  'B'   */
    0b1100101, /*  'C'   */
    0b0011111, /*  'D'   */
    0b1101101, /*  'E'   */
    0b1101100, /*  'F'   */
    0b1100111, /*  'G'   */
    0b0111110, /*  'H'   */
    0b0100100, /*  'I'   */
    0b0000010, /*  'J'   */
    0b0101101, /*  'K'   */
    0b0100101, /*  'L'   */
    0b1110110, /*  'M'   */
    0b1110110, /*  'N'   */
    0b1110111, /*  'O'   */
    0b1111100, /*  'P'   */
    0b1111010, /*  'Q'   */
    0b1111110, /*  'R'   */
    0b1101011, /*  'S'   */
    0b1100100, /*  'T'   */
    0b0110111, /*  'U'   */
    0b0110111, /*  'V'   */
    0b0110111, /*  'W'   */
    0b0111110, /*  'X'   */
    0b0111010, /*  'Y'   */
    0b1011101, /*  'Z'   */
];

pub fn segment_to_byte(segment: u8) -> u8 {
    let mut byte = 0;
    let data_offset = [2, 0, 4, 3, 1, 7, 5];
    for i in 0..7 {
        let segment_mask = 1 << (6 - i);
        if (segment & segment_mask) != 0 {
            byte |= 1 << data_offset[i];
        }
    }
    byte
}

pub struct Tnarx<'a> {
    ce: &'a mut OutputPin,
    ck: &'a mut OutputPin,
    di: &'a mut OutputPin,
    data: [u8; 14],
}

impl<'a> Tnarx<'a> {
    pub fn new(ce: &'a mut OutputPin, ck: &'a mut OutputPin, di: &'a mut OutputPin) -> Tnarx<'a> {
        let data = [0; 14];
        Self { ce, ck, di, data }
    }
    fn write_bits(&mut self, data_s: usize) {
        /* assuming length 7 */
        self.ck.set_low();
        self.di.set_low();
        self.ce.set_high();
        for c in self.data[data_s..data_s + 7].iter() {
            // This is directly taken from the arduino implementation
            let mut mask = 128;
            while mask != 0 {
                if (mask & *c) != 0 {
                    self.di.set_high();
                } else {
                    self.di.set_low();
                }
                self.ck.set_high();
                mask >>= 1;
                self.ck.set_low();
            }
        }
        self.di.set_low();
        self.ce.set_low();
    }

    pub fn flush(&mut self) {
        self.data[6] &= 248;
        self.data[6] |= 4;
        self.data[7 + 6] &= 248;
        self.data[7 + 6] |= 1;
        self.write_bits(0);
        self.write_bits(7);
    }

    pub fn erase(&mut self) {
        for c in self.data.iter_mut() {
            *c = 0;
        }
    }
    fn set(&mut self, segment: u8, mut position: usize) {
        if position >= 6 {
            position += 1;
        }
        self.data[12 - position] = segment_to_byte(segment);
    }
}

use core::fmt::Result;
use core::fmt::Write;

impl<'a> Write for Tnarx<'a> {
    fn write_str(&mut self, s: &str) -> Result {
        let s = if s.len() > 12 { &s[0..12] } else { s };
        let mut pos = 0;
        for c in s.chars() {
            let c = c as u8;
            let b = if c >= b'A' && c < b'A' + LOOKUP_TABLE.len() as u8 {
                LOOKUP_TABLE[(c - b'A') as usize]
            } else {
                0b1011100
            };
            self.set(b, pos);
            pos += 1;
        }
        Ok(())
    }
}
