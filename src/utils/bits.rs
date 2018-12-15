// Bit manipulation module

use crate::utils::bits;

// Remove everything not in the bit mask
pub fn mask(byte: u8, mask: u8) -> u8 {
    byte & mask
}

// Copy the source into the destination filtering by the mask
pub fn copy(dest: u8, source: u8, mask: u8) -> u8 {
    bits::mask(dest, !mask) | bits::mask(source, mask)
}

// Set and clear bits
pub fn set(byte: u8, index: u8) -> u8 { byte | (1 << index) }

pub fn clear(data: u8, index: u8) -> u8 {
    data & !(1 << index)
}

// Check a bit
pub fn is_set(byte: u8, index: u8) -> bool { (byte & (1 << index)) != 0 }

pub fn is_clear(byte: u8, index: u8) -> bool { !bits::is_set(byte, index) }

// Return an array of eight bytes interlacing between high and low.
pub fn interlace(low: u8, high: u8, index: u8) -> u8 {
    let high = high >> (7 - index);
    let low = low >> (7 - index);

    ((high & 0x01) << 1) | (low & 0x01)
}

// Create a word based on the two bytes
pub fn word(high: u8, low: u8) -> u16 { (u16::from(high) << 8) | u16::from(low) }

// Get the high byte of a word
pub fn low(word: u16) -> u8 { word as u8 }

pub fn low_word(word: u16) -> u16 { word & 0x00ff }

// Get the low byte of a word
pub fn high(word: u16) -> u8 { (word >> 8) as u8 }

pub fn high_word(word: u16) -> u16 { word & 0xff00 }

// Set the low byte of a word
pub fn set_low(word: u16, byte: u8) -> u16 {
    (word & 0xff00) | u16::from(byte)
}

// Set the high byte of a word
pub fn set_high(word: u16, byte: u8) -> u16 {
    (word & 0x00ff) | (u16::from(byte) << 8)
}