use std::fmt::{self, Debug};

static I_SHIFT: u8 = 49 + 7;
static J_SHIFT: u8 = 49;
static BOARD_MASK: u64 = 0x1ffffffffffff;
static IJ_MASK: u64 = 0x7ffe000000000000;

// A mask for the bits that do not get "shifted out" when changing the offset's i value.
// The index into this array is (offset_i_new - offset_i + 7), clamped to (0, 14).
static SHIFT_MASK_I: [u64; 15] = [
    0b0000000000000000000000000000000000000000000000000,
    0b0000000000000000000000000000000000000000001111111,
    0b0000000000000000000000000000000000011111111111111,
    0b0000000000000000000000000000111111111111111111111,
    0b0000000000000000000001111111111111111111111111111,
    0b0000000000000011111111111111111111111111111111111,
    0b0000000111111111111111111111111111111111111111111,
    0b1111111111111111111111111111111111111111111111111,
    0b1111111111111111111111111111111111111111110000000,
    0b1111111111111111111111111111111111100000000000000,
    0b1111111111111111111111111111000000000000000000000,
    0b1111111111111111111110000000000000000000000000000,
    0b1111111111111100000000000000000000000000000000000,
    0b1111111000000000000000000000000000000000000000000,
    0b0000000000000000000000000000000000000000000000000,
];

// A mask for the bits that do not get "shifted out" when changing the offset's j value.
// The index into this array is (offset_j_new - offset_j + 7), clamped to (0, 14).
static SHIFT_MASK_J: [u64; 15] = [
    0b0000000000000000000000000000000000000000000000000,
    0b0000001000000100000010000001000000100000010000001,
    0b0000011000001100000110000011000001100000110000011,
    0b0000111000011100001110000111000011100001110000111,
    0b0001111000111100011110001111000111100011110001111,
    0b0011111001111100111110011111001111100111110011111,
    0b0111111011111101111110111111011111101111110111111,
    0b1111111111111111111111111111111111111111111111111,
    0b1111110111111011111101111110111111011111101111110,
    0b1111100111110011111001111100111110011111001111100,
    0b1111000111100011110001111000111100011110001111000,
    0b1110000111000011100001110000111000011100001110000,
    0b1100000110000011000001100000110000011000001100000,
    0b1000000100000010000001000000100000010000001000000,
    0b0000000000000000000000000000000000000000000000000,
];

/// A [`Copy`] board representation that stores only a single
/// bit per field.
///
/// Intended for storing all cards of a particular suit, for
/// efficient line detection.
///
/// It can be converted back into a list of coordinate pairs by
/// means of its [`IntoIterator`] instance.
///
/// Note that its "mutating" methods return a new object instead of really mutating.
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Clone, Copy)]
pub struct BitBoard {
    /// The low 49 bits are the board itself (7x7)
    /// The next highest 7 bits are the j offset.
    /// The next highest 7 bits are the i offset.
    /// The uppermost bit indicates whether the author of this
    /// code is cool as fuck (it is set to 0 if true).
    ///
    /// How can we store a i8 in 7 bits? Well, the actual range
    /// of card coordinates is much lower than the range of an
    /// i8. An upper bound on the actual range is -52 to 52, because
    /// that's how many cards there are in the game, and you cannot
    /// "move" the board towards any direction more than by one per
    /// card played.
    /// So, all the numbers in [-64i8, -1i8] start with the bits 11
    /// and all the numbers in [0, 63i8] start with the bits 00.
    /// Therefore, compression works by removing the highest bit,
    /// and adding it back when reading.
    ///
    /// How do (i, j) coordinates map to bits in the board?
    /// (i, j) is represented as the bit number  (i * 7 + j), counted from
    /// the least significant bit. So if you lay out a number like
    /// 0b0000000000000011111111111111111111111111111111111 in blocks of 7
    /// from least significant to most significant bit
    /// (which is also what the Debug impl does) like so:
    ///
    /// ```text
    /// 1 1 1 1 1 1 1
    /// 1 1 1 1 1 1 1
    /// 1 1 1 1 1 1 1
    /// 1 1 1 1 1 1 1
    /// 1 1 1 1 1 1 1
    /// 0 0 0 0 0 0 0
    /// 0 0 0 0 0 0 0
    /// ```
    /// then this 2D array effectively has a coordinate system that has i going from the
    /// top (0) to the bottom (6), and j going from the left (0) to the right (6).
    bits: u64,
}

// !!!!!! NOTE: Keep in sync with pymethods impl block !!!!!!
impl BitBoard {
    // This is only crate-public because it is valid only for a certain range of i and j
    pub(crate) fn empty_board_centered_at(i: i8, j: i8) -> Self {
        debug_assert!(i >= -52);
        debug_assert!(j >= -52);
        debug_assert!(i <= 52);
        debug_assert!(j <= 52);
        // This makes use of a really nice property:
        // When we place the first coordinate in the center of
        // the 7x7 area that is modeled, then no matter where the
        // remaining cards are, if there is a 4x4 bbox enclosing
        // all the cards, it will fit within the 7x7 area.
        let offset_i = i - 3;
        let offset_j = j - 3;
        Self {
            bits: encode_offset(offset_i, offset_j),
        }
    }

    /// Set the bit at the specified location to `true`.
    ///
    /// This is only valid for coordinates in a 7x7 area around the center of the `BitBoard`.
    /// Other indices will cause a panic in debug mode.
    #[must_use]
    pub(crate) fn insert(self, i: i8, j: i8) -> Self {
        let idx = self.arr_idx(i, j);
        Self {
            bits: self.bits | (1u64 << idx),
        }
    }

    #[must_use]
    pub(crate) fn insert_area(self, i_min: i8, j_min: i8, i_max: i8, j_max: i8) -> Self {
        let mut bits = self.bits;
        let (min_local_i, min_local_j) = self.local_coords(i_min, j_min);
        let (max_local_i, max_local_j) = self.local_coords(i_max, j_max);
        for i in min_local_i..=max_local_i {
            for j in min_local_j..=max_local_j {
                bits |= 1u64 << (i * 7 + j);
            }
        }
        Self { bits }
    }

    #[must_use]
    pub(crate) fn remove(self, i: i8, j: i8) -> Self {
        let idx = self.arr_idx(i, j);
        Self {
            bits: self.bits & !(1u64 << idx),
        }
    }

    pub fn contains(self, i: i8, j: i8) -> bool {
        let (offset_i, offset_j) = self.offset();
        let i_local = if let Some(i_local) = i.checked_sub(offset_i) {
            i_local
        } else {
            return false;
        };
        let j_local = if let Some(j_local) = j.checked_sub(offset_j) {
            j_local
        } else {
            return false;
        };
        if i_local >= 7 || j_local >= 7 {
            return false;
        }
        let idx = i_local * 7 + j_local;
        self.bits & (1u64 << idx) != 0
    }

    pub fn is_empty(self) -> bool {
        (self.bits & BOARD_MASK) == 0
    }

    pub fn num_entries(self) -> u32 {
        (self.bits & BOARD_MASK).count_ones()
    }

    pub(crate) fn center(self) -> (i8, i8) {
        let (offset_i, offset_j) = self.offset();
        (offset_i + 3, offset_j + 3)
    }

    pub(crate) fn recenter_to(self, new_center: (i8, i8)) -> BitBoard {
        let (offset_i, offset_j) = self.offset();
        let (new_offset_i, new_offset_j) = (new_center.0 - 3, new_center.1 - 3);
        debug_assert!((offset_i - new_offset_i).abs() < 4);
        debug_assert!((offset_j - new_offset_j).abs() < 4);

        let board_bits = self.bits & BOARD_MASK;

        // Imagine the 49 board bits like this (top left is lowest bit, bottom right highest, and
        // i selects the row, j the column):
        //
        // . . . . . . .
        // . . . . . . .
        // . . . . 1 . .
        // . . 1 x . 1 .
        // . . . . 1 n .
        // . . . . . . .
        // . . . . . . .
        //
        // The center is marked with x, cards are marked with 1 (since they are 1s in the bitset),
        // and the new center is marked with n. We need to shift n to land on x.
        // It's important to realize that there will never be any wraparound in the sense that a
        // 1 lands on the other side of the board, as long as the new center is a valid center for
        // a bitboard that contains the same cards.

        let diff = (offset_i - new_offset_i) * 7 + (offset_j - new_offset_j);
        let board_bits_shifted = if diff > 0 {
            // Since the new center has lower coordinates, the local coordinates will have
            // higher coordinates => shift left
            board_bits << diff
        } else {
            board_bits >> diff.abs()
        };
        // No ones should have gotten lost
        debug_assert_eq!(
            board_bits.count_ones(),
            (board_bits_shifted & BOARD_MASK).count_ones()
        );
        let offset_bits = encode_offset(new_offset_i, new_offset_j);
        Self {
            bits: offset_bits | board_bits_shifted,
        }
    }

    pub(crate) fn shift_lossy(self, new_center: (i8, i8)) -> BitBoard {
        assert!(new_center.0 >= -52);
        assert!(new_center.1 >= -52);
        assert!(new_center.0 <= 52);
        assert!(new_center.1 <= 52);

        let (offset_i, offset_j) = self.offset();
        let (new_offset_i, new_offset_j) = (new_center.0 - 3, new_center.1 - 3);
        let (diff_i, diff_j) = (new_offset_i - offset_i, new_offset_j - offset_j);
        let mask_i = SHIFT_MASK_I[(diff_i + 7).clamp(0, 14) as usize];
        let mask_j = SHIFT_MASK_J[(diff_j + 7).clamp(0, 14) as usize];
        let valid_bits = self.bits & mask_i & mask_j;
        let shift_by = diff_i * 7 + diff_j;
        let bits_shifted = if shift_by > 0 {
            valid_bits >> shift_by
        } else {
            valid_bits << shift_by.abs()
        };
        let offset_bits = encode_offset(new_offset_i, new_offset_j);
        Self {
            bits: offset_bits | bits_shifted,
        }
    }

    /// Compute the difference to another `BitBoard`.
    ///
    /// Both boards must be centered around the same point, otherwise this
    /// function will panic.
    #[must_use]
    pub fn difference(self, other: BitBoard) -> BitBoard {
        assert_eq!(self.bits & IJ_MASK, other.bits & IJ_MASK);
        Self {
            bits: self.bits & !(other.bits & BOARD_MASK),
        }
    }

    #[must_use]
    pub(crate) fn detect_central_lines(self) -> BitBoard {
        let mut line_bits = 0;
        // These patterns are lines on the 7x7 board - horizontal, vertical, and two diagonal.
        // They already are zero outside of BOARD_MASK, so self.bits & pattern is the same as
        // (self.bits & BOARD_MASK) & pattern.
        for pattern in [
            0xfe00000u64,
            0x204081020408u64,
            0x1010101010101u64,
            0x41041041040u64,
        ] {
            let pattern_intersect = self.bits & pattern;
            debug_assert!(pattern_intersect.count_ones() <= 4);
            if pattern_intersect.count_ones() == 4 {
                line_bits |= pattern_intersect;
            }
        }
        Self {
            bits: (self.bits & IJ_MASK) | line_bits,
        }
    }

    fn local_coords(self, i: i8, j: i8) -> (u8, u8) {
        let (offset_i, offset_j) = self.offset();
        debug_assert!(i >= offset_i);
        debug_assert!(j >= offset_j);
        debug_assert!(i - offset_i < 7);
        debug_assert!(j - offset_j < 7);
        let i_local = (i - offset_i) as u8;
        let j_local = (j - offset_j) as u8;
        (i_local, j_local)
    }

    fn arr_idx(self, i: i8, j: i8) -> u8 {
        let (i_local, j_local) = self.local_coords(i, j);
        i_local * 7 + j_local
    }

    fn offset(self) -> (i8, i8) {
        decode_offset(self.bits)
    }
}

fn decode_offset(bits: u64) -> (i8, i8) {
    // The highest bit of i_compressed is garbage and needs
    // to be replaced with the second-highest bit.
    let offset_i_compressed = 0b01111111i8 & (bits >> I_SHIFT) as i8;
    let offset_i = offset_i_compressed | ((offset_i_compressed & 0b01000000i8) << 1);
    let offset_j_compressed = 0b01111111i8 & (bits >> J_SHIFT) as i8;
    let offset_j = offset_j_compressed | ((offset_j_compressed & 0b01000000i8) << 1);
    (offset_i, offset_j)
}

fn encode_offset(offset_i: i8, offset_j: i8) -> u64 {
    let offset_i_bits = u64::from(offset_i as u8 & 0b01111111u8) << I_SHIFT;
    let offset_j_bits = u64::from(offset_j as u8 & 0b01111111u8) << J_SHIFT;
    offset_i_bits | offset_j_bits
}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let digits = format!("{:049b}", self.bits & BOARD_MASK);
        let mut s = String::with_capacity(49 * 2);
        for (idx, c) in digits.chars().rev().enumerate() {
            s.push(c);
            if idx % 7 == 6 {
                s.push('\n');
            } else {
                s.push(' ');
            }
        }
        write!(f, "{}", s)
    }
}

/// Iterator produced by [`BitBoard::into_iter()`].
pub struct BitBoardIter {
    bitboard: BitBoard,
}

impl IntoIterator for BitBoard {
    type Item = (i8, i8);

    type IntoIter = BitBoardIter;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardIter { bitboard: self }
    }
}

impl Iterator for BitBoardIter {
    type Item = (i8, i8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitboard.is_empty() {
            None
        } else {
            // This cast is safe, as the max value for trailing_zeros is 64
            let idx: i8 = self.bitboard.bits.trailing_zeros() as i8;
            let (offset_i, offset_j) = self.bitboard.offset();
            // Clear the flag corresponding to this coordinate
            self.bitboard.bits ^= 1u64 << idx;
            Some((offset_i + idx / 7, offset_j + idx % 7))
        }
    }
}

#[cfg(feature = "python")]
mod python {
    use pyo3::pymethods;

    use super::*;

    #[pymethods]
    impl BitBoard {
        #[pyo3(name = "contains")]
        fn py_contains(&self, i: i8, j: i8) -> bool {
            self.contains(i, j)
        }

        #[pyo3(name = "is_empty")]
        fn py_is_empty(&self) -> bool {
            self.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::quickcheck;

    use super::*;

    quickcheck! {
        fn offset_compression(i: i8, j: i8) -> bool {
            // Restrict i and j to the range [-52, 52]
            let i = i % 53;
            let j = j % 53;
            BitBoard::empty_board_centered_at(i, j).offset() == (i - 3, j - 3)
        }
    }

    #[test]
    fn recenter() {
        let bb = BitBoard::empty_board_centered_at(12, 30)
            .insert(12, 30)
            .insert(12, 33)
            .insert(15, 30);
        assert_eq!(bb.bits, bb.recenter_to((15, 33)).recenter_to((12, 30)).bits);
    }

    #[test]
    fn shift() {
        let bb = BitBoard::empty_board_centered_at(12, 30)
            .insert(12, 30)
            .insert(12, 33)
            .insert(15, 30);
        assert_eq!(bb.bits, bb.shift_lossy((15, 33)).shift_lossy((12, 30)).bits);
    }
}
