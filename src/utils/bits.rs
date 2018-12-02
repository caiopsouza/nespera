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

// Check if a bit is set
pub fn is_set(data: u8, index: u8) -> bool {
    (data & (1 << index)) != 0
}

// Set the low byte of a word
pub fn set_low(word: u16, byte: u8) -> u16 {
    (word & 0xff00) | (byte as u16)
}

// Set the high byte of a word
pub fn set_high(word: u16, byte: u8) -> u16 {
    (word & 0x00ff) | ((byte as u16) << 8)
}
