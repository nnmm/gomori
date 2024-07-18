mod bbox;
mod compact_field;

use pyo3::pymethods;

use crate::{Board, BoundingBox};

#[pymethods]
impl Board {
    #[pyo3(name = "bbox")]
    pub(in crate::python) fn py_bbox(&self) -> BoundingBox {
        self.bbox()
    }
}
