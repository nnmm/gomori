/// Macro to help with defining bitset types
macro_rules! bitset_traits {
    ($name:ident) => {
        impl std::ops::BitAnd for $name {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output {
                Self {
                    bits: self.bits & rhs.bits,
                }
            }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self {
                    bits: self.bits | rhs.bits,
                }
            }
        }

        impl std::ops::BitXor for $name {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self::Output {
                Self {
                    bits: self.bits ^ rhs.bits,
                }
            }
        }

        impl std::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.bits &= rhs.bits;
            }
        }

        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.bits |= rhs.bits;
            }
        }

        impl std::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.bits ^= rhs.bits;
            }
        }

        impl std::ops::Not for $name {
            type Output = Self;

            fn not(self) -> Self::Output {
                Self { bits: !self.bits }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self { bits: 0 }
            }
        }
    };
}
pub(crate) use bitset_traits;
