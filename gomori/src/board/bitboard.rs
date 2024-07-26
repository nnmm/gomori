use std::fmt::{self, Debug};

const I_SHIFT: u8 = 49 + 7;
const J_SHIFT: u8 = 49;
const BOARD_MASK: u64 = 0x1ffffffffffff;
const OFFSET_MASK: u64 = 0x7ffe000000000000;

/// A compact board representation that stores only a single
/// bit per field, equivalent to a set of coordinates.
///
/// Every `BitBoard` is obtained from a [`Board`](crate::Board), and can, for instance, store
/// [which fields on the board have a visible diamond card on them](crate::Board::diamonds).
/// So a `BitBoard` is kind of like a "view" into a particular `Board`.
///
/// Two `BitBoard`s can be combined with operations like [`&`](std::ops::BitAnd),
/// [`|`](std::ops::BitOr`), or [`difference()`](BitBoard::difference).
/// It's important that when two `BitBoard`s are combined, they must both originate[^note] from the same `Board`.
/// The reason is that the area represented by `BitBoard`s from different `Board`s may not be identical, so some
/// contents of one board may be unrepresentable in the other `BitBoard`.
/// This is only checked with a debug assertion.
///
/// [^note]: "`BitBoard` X originates from `Board` B" means that that X was returned by some method of B,
///     or that X is the result of combining/processing `BitBoard`s that originate from X.
///
/// It can be converted back into a list of coordinate pairs by
/// means of its [`IntoIterator`] instance.
///
/// # Note on immutability
///
/// This is an immutable type, so its "mutating" methods return a
/// new value instead of really mutating in-place (except for `std::ops::BitXxxAssign` trait methods).
/// It is also [`Copy`], so a value is not consumed by methods with `self` receiver.
///
/// # Implementation
///
/// Internally, a `BitBoard` is (1) an `(offset_i, offset_j)` coordinate pair and (2) a 49-bit-bitset. The bitset encodes an 7 x 7 area with one bit per field, like so:
///
///
/// ```text
/// 1 0 1 1 0 0 1
/// 1 1 1 0 1 0 1
/// 0 1 1 0 0 1 0
/// 0 0 1 1 1 1 0
/// 0 1 1 0 1 0 1
/// 0 0 0 1 0 1 1
/// 0 0 0 1 0 0 1
/// ```
///
/// The offset is added to every local `(i, j)` coordinate in that field (the local `i` and `j` both range from `0` to `6`) to obtain the true `(i, j)` coordinate.
///
/// Every valid board would fit in a 4 x 4 area, so why 7 x 7? One reason is that with a 7 x 7 board,
/// we can be sure that not only the board itself can be represented, but also the next card, as long as it is in the board's [playable area](crate::Board::playable_area).
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
    /// 0b1100000000000011111111111111111111111111111111111 in blocks of 7
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
    /// 0 0 0 0 0 1 1
    /// ```
    /// then this 2D array effectively has a coordinate system that has i going from the
    /// top (0) to the bottom (6), and j going from the left (0) to the right (6).
    bits: u64,
}

// !!!!!! NOTE: Keep in sync with pymethods impl block !!!!!!
impl BitBoard {
    // This is only crate-public because it is valid only for a certain range of i and j
    pub(crate) fn empty_board_centered_at((i, j): (i8, i8)) -> Self {
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

    /// Set the bit for the specified coordinate to `true`.
    ///
    /// This function must only be used with coordinates in the underlying board's [`playable_area`](crate::Board::playable_area).
    /// Other coordinates may exceed the 7x7 area stored in the `BitBoard`, and that will cause a panic in debug mode.
    /// In release mode, no checks are performed, and data
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

    /// Computes the difference to another `BitBoard`.
    #[must_use]
    pub fn difference(self, other: BitBoard) -> BitBoard {
        debug_assert_eq!(self.bits & OFFSET_MASK, other.bits & OFFSET_MASK);
        Self {
            bits: self.bits & !(other.bits & BOARD_MASK),
        }
    }

    /// Checks whether there are any horizontal, vertical or diagonal lines of length 4
    /// passing through the specified point (in a 7 x 7 area centered on the point).
    ///
    /// Any lines that are found are returned in a new `BitBoard`. The result is therefore
    /// a subset of the input.
    ///
    /// Only valid for point coordinates in the range `[-52, 52]`.
    #[must_use]
    pub fn lines_going_through_point(self, point_i: i8, point_j: i8) -> BitBoard {
        debug_assert!(point_i >= -52);
        debug_assert!(point_j >= -52);
        debug_assert!(point_i <= 52);
        debug_assert!(point_j <= 52);

        let (offset_i, offset_j) = self.offset();
        let delta = (point_i - offset_i - 3, point_j - offset_j - 3);

        let mut line_bits = 0;
        // These patterns are lines on the 7x7 board - horizontal, vertical, and two diagonal.
        for pattern in [
            0xfe00000u64,
            0x204081020408u64,
            0x1010101010101u64,
            0x41041041040u64,
        ] {
            let pattern_intersect = self.bits & shift_2d_lossy(pattern, delta);
            debug_assert!(pattern_intersect.count_ones() <= 4);
            if pattern_intersect.count_ones() == 4 {
                line_bits |= pattern_intersect;
            }
        }
        Self {
            bits: self.bits & OFFSET_MASK | line_bits,
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

// A 2D shift can be implemented as a mask + a bitshift.
// If we only did a bitshift without masking, then we'd get artifacts from bits wrapping around.
fn shift_2d_lossy(bits: u64, (delta_i, delta_j): (i8, i8)) -> u64 {
    // A mask for the bits that do not get "shifted out" by moving all points by delta_i along the i axis.
    static SHIFT_MASK_I: [u64; 15] = [
        0b0000000000000000000000000000000000000000000000000,
        0b1111111000000000000000000000000000000000000000000,
        0b1111111111111100000000000000000000000000000000000,
        0b1111111111111111111110000000000000000000000000000,
        0b1111111111111111111111111111000000000000000000000,
        0b1111111111111111111111111111111111100000000000000,
        0b1111111111111111111111111111111111111111110000000,
        0b1111111111111111111111111111111111111111111111111,
        0b0000000111111111111111111111111111111111111111111,
        0b0000000000000011111111111111111111111111111111111,
        0b0000000000000000000001111111111111111111111111111,
        0b0000000000000000000000000000111111111111111111111,
        0b0000000000000000000000000000000000011111111111111,
        0b0000000000000000000000000000000000000000001111111,
        0b0000000000000000000000000000000000000000000000000,
    ];

    static SHIFT_MASK_J: [u64; 15] = [
        0b0000000000000000000000000000000000000000000000000,
        0b1000000100000010000001000000100000010000001000000,
        0b1100000110000011000001100000110000011000001100000,
        0b1110000111000011100001110000111000011100001110000,
        0b1111000111100011110001111000111100011110001111000,
        0b1111100111110011111001111100111110011111001111100,
        0b1111110111111011111101111110111111011111101111110,
        0b1111111111111111111111111111111111111111111111111,
        0b0111111011111101111110111111011111101111110111111,
        0b0011111001111100111110011111001111100111110011111,
        0b0001111000111100011110001111000111100011110001111,
        0b0000111000011100001110000111000011100001110000111,
        0b0000011000001100000110000011000001100000110000011,
        0b0000001000000100000010000001000000100000010000001,
        0b0000000000000000000000000000000000000000000000000,
    ];

    // Larger values will get clamped to the ends, where all bits will be masked out.
    let mask_i = SHIFT_MASK_I[(delta_i + 7).clamp(0, 14) as usize];
    let mask_j = SHIFT_MASK_J[(delta_j + 7).clamp(0, 14) as usize];
    let valid_bits = bits & mask_i & mask_j;
    let shift_by = delta_i * 7 + delta_j;
    if shift_by > 0 {
        valid_bits << shift_by.min(63)
    } else {
        valid_bits >> shift_by.abs().min(63)
    }
}

impl std::ops::BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        debug_assert_eq!(self.bits & OFFSET_MASK, rhs.bits & OFFSET_MASK);
        Self {
            bits: self.bits & rhs.bits,
        }
    }
}

impl std::ops::BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        debug_assert_eq!(self.bits & OFFSET_MASK, rhs.bits & OFFSET_MASK);
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl std::ops::BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        debug_assert_eq!(self.bits & OFFSET_MASK, rhs.bits & OFFSET_MASK);
        Self {
            bits: self.bits ^ (rhs.bits & BOARD_MASK),
        }
    }
}

impl std::ops::BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        debug_assert_eq!(self.bits & OFFSET_MASK, rhs.bits & OFFSET_MASK);
        self.bits &= rhs.bits;
    }
}

impl std::ops::BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        debug_assert_eq!(self.bits & OFFSET_MASK, rhs.bits & OFFSET_MASK);
        self.bits |= rhs.bits;
    }
}

impl std::ops::BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        debug_assert_eq!(self.bits & OFFSET_MASK, rhs.bits & OFFSET_MASK);
        self.bits ^= rhs.bits & BOARD_MASK;
    }
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

        #[pyo3(name = "difference")]
        fn py_difference(&self, other: BitBoard) -> BitBoard {
            self.difference(other)
        }

        #[pyo3(name = "lines_going_through_point")]
        fn py_lines_going_through_point(&self, point_i: i8, point_j: i8) -> BitBoard {
            self.lines_going_through_point(point_i, point_j)
        }

        fn __len__(&self) -> usize {
            self.num_entries() as usize
        }

        fn __bool__(&self) -> bool {
            !self.is_empty()
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
            BitBoard::empty_board_centered_at((i, j)).offset() == (i - 3, j - 3)
        }
    }

    #[test]
    fn shift_far() {
        let bb = BitBoard::empty_board_centered_at((12, 30))
            .insert(11, 32)
            .insert(12, 30)
            .insert(12, 33)
            .insert(15, 30);
        // None of the coordinates on the board are representable with that offset.
        let bits_shifted = shift_2d_lossy(bb.bits, (-12, -30));
        assert_eq!(bits_shifted, 0);
    }

    #[test]
    fn detect_line() {
        let bb = BitBoard::empty_board_centered_at((10, 10))
            .insert(8, 11)
            .insert(11, 11)
            .insert(12, 11)
            .insert(13, 11);
        assert_eq!(
            Vec::from_iter(bb.lines_going_through_point(11, 11)),
            Vec::from_iter(bb)
        );
        assert_eq!(
            Vec::from_iter(bb.lines_going_through_point(9, 9)),
            Vec::new()
        );
    }
}
