use pyo3::prelude::*;

use pyo3::create_exception;

create_exception!(
    gomori,
    IllegalCardPlayed,
    pyo3::exceptions::PyException,
    "Describes why the card cannot be played."
);

/// A Python module implemented in Rust.
#[pymodule]
fn gomori(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("IllegalCardPlayed", py.get_type::<IllegalCardPlayed>())?;
    m.add_class::<::gomori::Card>()?;
    m.add_class::<::gomori::Rank>()?;
    m.add_class::<::gomori::Suit>()?;
    m.add_class::<::gomori::CardsSet>()?;
    m.add_class::<::gomori::CompactField>()?;
    Ok(())
}
