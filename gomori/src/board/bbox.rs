/// A 2D area represented by a min + max coordinate pair.
///
/// The two coordinates form an _inclusive_ 2D range, i.e. unlike in a
/// half-open range, it's possible for a point with `i == i_max`
/// to be contained in the area.
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub i_min: i8,
    pub j_min: i8,
    pub i_max: i8,
    pub j_max: i8,
}

// !!!!!! NOTE: Keep in sync with pymethods impl block !!!!!!
impl BoundingBox {
    pub fn contains(&self, i: i8, j: i8) -> bool {
        i >= self.i_min && j >= self.j_min && i <= self.i_max && j <= self.j_max
    }

    pub fn singleton(i: i8, j: i8) -> Self {
        Self {
            i_min: i,
            j_min: j,
            i_max: i,
            j_max: j,
        }
    }

    pub fn from_coordinates_iter(mut iter: impl Iterator<Item = (i8, i8)>) -> Option<Self> {
        let (i0, j0) = iter.next()?;
        let (mut i_min, mut j_min, mut i_max, mut j_max) = (i0, j0, i0, j0);
        for (i, j) in iter {
            i_min = i_min.min(i);
            i_max = i_max.max(i);
            j_min = j_min.min(j);
            j_max = j_max.max(j);
        }
        Some(Self {
            i_min,
            j_min,
            i_max,
            j_max,
        })
    }

    /// Expands the bounding box to cover point `(i, j)`.
    pub fn update(&mut self, i: i8, j: i8) {
        self.i_min = self.i_min.min(i);
        self.i_max = self.i_max.max(i);
        self.j_min = self.j_min.min(j);
        self.j_max = self.j_max.max(j);
    }
}

#[cfg(feature = "python")]
mod python {
    use pyo3::pymethods;

    use super::*;
    #[pymethods]
    impl BoundingBox {
        #[new]
        #[pyo3(signature = (*, i_min, j_min, i_max, j_max))]
        fn py_new(i_min: i8, j_min: i8, i_max: i8, j_max: i8) -> Self {
            Self {
                i_min,
                j_min,
                i_max,
                j_max,
            }
        }

        #[pyo3(name = "contains")]
        fn py_contains(&self, i: i8, j: i8) -> bool {
            self.contains(i, j)
        }

        #[pyo3(name = "update")]
        fn py_update(&mut self, i: i8, j: i8) {
            self.update(i, j)
        }
    }
}
