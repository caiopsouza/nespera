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

// Check if a bit is set
pub fn is_set(byte: u8, index: u8) -> bool {
    (byte & (1 << index)) != 0
}

// Create a word based on the two bytes
pub fn word(high: u8, low: u8) -> u16 { ((high as u16) << 8) | (low as u16) }

// Get the high byte of a word
pub fn low(word: u16) -> u8 { word as u8 }

// Get the low byte of a word
pub fn high(word: u16) -> u8 { (word >> 8) as u8 }

// Set the low byte of a word
pub fn set_low(word: u16, byte: u8) -> u16 {
    (word & 0xff00) | (byte as u16)
}

// Set the high byte of a word
pub fn set_high(word: u16, byte: u8) -> u16 {
    (word & 0x00ff) | ((byte as u16) << 8)
}