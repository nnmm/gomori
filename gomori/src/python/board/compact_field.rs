use pyo3::pymethods;

use crate::{Card, CardsSet, CardsSetIter, CompactField, Field};

#[pymethods]
impl CardsSet {
    #[new]
    #[pyo3(signature = (cards=vec![]))]
    fn py_new(cards: Vec<Card>) -> Self {
        Self::from_iter(cards)
    }

    fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    fn __len__(&self) -> usize {
        self.len() as usize
    }

    fn __contains__(&self, card: Card) -> bool {
        self.contains(card)
    }

    fn __iter__(&self) -> CardsSetIter {
        self.into_iter()
    }

    fn __repr__(&self) -> String {
        let card_reprs: Vec<_> = self.into_iter().map(|c| c.__repr__()).collect();
        format!("CardsSet([{}])", card_reprs.join(", "))
    }

    #[getter]
    #[pyo3(name = "is_empty")]
    fn py_is_empty(&self) -> bool {
        self.is_empty()
    }

    #[pyo3(name = "insert")]
    fn py_insert(&mut self, card: Card) {
        *self = self.insert(card);
    }
}

#[pymethods]
impl CardsSetIter {
    fn __iter__(&self) -> Self {
        *self
    }

    fn __next__(&mut self) -> Option<Card> {
        self.next()
    }
}

#[pymethods]
impl CompactField {
    fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    #[pyo3(name = "is_empty")]
    fn py_is_empty(&self) -> bool {
        self.is_empty()
    }

    #[getter]
    #[pyo3(name = "top_card")]
    fn py_top_card(&self) -> Option<Card> {
        self.top_card()
    }

    #[pyo3(name = "can_place_card")]
    fn py_can_place_card(&self, card: Card) -> bool {
        self.can_place_card(card)
    }

    #[getter]
    #[pyo3(name = "num_hidden_cards")]
    fn py_num_hidden_cards(&self) -> u32 {
        self.num_hidden_cards()
    }

    #[pyo3(name = "turn_face_down")]
    fn py_turn_face_down(&self) -> Self {
        self.turn_face_down()
    }

    #[getter]
    #[pyo3(name = "hidden_cards")]
    fn py_hidden_cards(&self) -> CardsSet {
        self.hidden_cards()
    }

    #[pyo3(name = "into_field")]
    fn py_into_field(&self, i: i8, j: i8) -> Field {
        self.into_field(i, j)
    }
}
