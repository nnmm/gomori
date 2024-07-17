use pyo3::pymethods;

use crate::{Card, CardsSet, CardsSetIter, CompactField, Field};

#[pymethods]
impl CardsSet {
    #[new]
    #[pyo3(signature = (cards=vec![]))]
    pub(in crate::python) fn py_new(cards: Vec<Card>) -> Self {
        Self::from_iter(cards)
    }

    pub(in crate::python) fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    pub(in crate::python) fn __len__(&self) -> usize {
        self.len() as usize
    }

    pub(in crate::python) fn __contains__(&self, card: Card) -> bool {
        self.contains(card)
    }

    pub(in crate::python) fn __iter__(&self) -> CardsSetIter {
        self.into_iter()
    }

    pub(in crate::python) fn __repr__(&self) -> String {
        let card_reprs: Vec<_> = self.into_iter().map(|c| c.__repr__()).collect();
        format!("CardsSet([{}])", card_reprs.join(", "))
    }

    pub(in crate::python) fn __and__(&self, other: CardsSet) -> CardsSet {
        *self & other
    }

    pub(in crate::python) fn __or__(&self, other: CardsSet) -> CardsSet {
        *self | other
    }

    pub(in crate::python) fn __xor__(&self, other: CardsSet) -> CardsSet {
        *self ^ other
    }

    pub(in crate::python) fn __invert__(&self) -> CardsSet {
        !*self
    }

    pub(in crate::python) fn __iand__(&mut self, other: CardsSet) {
        *self &= other
    }

    pub(in crate::python) fn __ior__(&mut self, other: CardsSet) {
        *self |= other
    }

    pub(in crate::python) fn __ixor__(&mut self, other: CardsSet) {
        *self ^= other
    }

    #[getter]
    #[pyo3(name = "is_empty")]
    pub(in crate::python) fn py_is_empty(&self) -> bool {
        self.is_empty()
    }

    #[pyo3(name = "insert")]
    pub(in crate::python) fn py_insert(&mut self, card: Card) {
        *self = self.insert(card);
    }
}

#[pymethods]
impl CardsSetIter {
    pub(in crate::python) fn __iter__(&self) -> Self {
        *self
    }

    pub(in crate::python) fn __next__(&mut self) -> Option<Card> {
        self.next()
    }
}

#[pymethods]
impl CompactField {
    pub(in crate::python) fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    #[pyo3(name = "is_empty")]
    pub(in crate::python) fn py_is_empty(&self) -> bool {
        self.is_empty()
    }

    #[getter]
    #[pyo3(name = "top_card")]
    pub(in crate::python) fn py_top_card(&self) -> Option<Card> {
        self.top_card()
    }

    #[pyo3(name = "can_place_card")]
    pub(in crate::python) fn py_can_place_card(&self, card: Card) -> bool {
        self.can_place_card(card)
    }

    #[getter]
    #[pyo3(name = "num_hidden_cards")]
    pub(in crate::python) fn py_num_hidden_cards(&self) -> u32 {
        self.num_hidden_cards()
    }

    #[pyo3(name = "turn_face_down")]
    pub(in crate::python) fn py_turn_face_down(&self) -> Self {
        self.turn_face_down()
    }

    #[getter]
    #[pyo3(name = "hidden_cards")]
    pub(in crate::python) fn py_hidden_cards(&self) -> CardsSet {
        self.hidden_cards()
    }

    #[pyo3(name = "into_field")]
    pub(in crate::python) fn py_into_field(&self, i: i8, j: i8) -> Field {
        self.into_field(i, j)
    }
}
