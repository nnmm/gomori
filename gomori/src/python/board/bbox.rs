use pyo3::pymethods;

use crate::BoundingBox;

#[pymethods]
impl BoundingBox {
    #[pyo3(name = "contains")]
    pub(in crate::python) fn py_contains(&self, i: i8, j: i8) -> bool {
        self.contains(i, j)
    }
}
