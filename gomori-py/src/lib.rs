use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn gomori(py: Python, m: &PyModule) -> PyResult<()> {
    m.add(
        "IllegalCardPlayed",
        py.get_type::<::gomori::IllegalCardPlayedException>(),
    )?;
    m.add_class::<::gomori::Card>()?;
    m.add_class::<::gomori::Rank>()?;
    m.add_class::<::gomori::Suit>()?;
    m.add_class::<::gomori::CardsSet>()?;
    m.add_class::<::gomori::CompactField>()?;
    m.add_class::<::gomori::BoundingBox>()?;
    m.add_class::<::gomori::BitBoard>()?;
    m.add_class::<::gomori::Board>()?;
    m.add_class::<::gomori::CardToPlay>()?;
    m.add_class::<::gomori::Field>()?;
    m.add_class::<::gomori::PyPlayCardCalculation>()?;
    Ok(())
}
