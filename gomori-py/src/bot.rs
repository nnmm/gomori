use gomori::{Board, Card, CardsSet, Color, Field, PlayTurnResponse};
use gomori_bot_utils::Bot;
use pyo3::{pyfunction, types::PyDict, Py, PyObject, Python};

struct PythonBot {
    bot: PyObject,
}

// TODO: Re-evaluate this whole design
impl gomori_bot_utils::Bot for PythonBot {
    fn new_game(&mut self, color: Color) {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            kwargs
                .set_item("color", Py::new(py, color).unwrap())
                .unwrap();
            self.bot
                .call_method(py, "new_game", (), Some(kwargs))
                .expect("Call to new_game() failed");
        })
    }

    fn play_first_turn(&mut self, cards: [Card; 5]) -> Card {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            kwargs
                .set_item("cards", cards.map(|card| Py::new(py, card).unwrap()))
                .unwrap();
            self.bot
                .call_method(py, "play_first_turn", (), Some(kwargs))
                .expect("Call to play_first_turn() failed")
                .extract(py)
                .expect("play_first_turn() returned wrong type")
        })
    }

    fn play_turn(
        &mut self,
        cards: [Card; 5],
        fields: Vec<Field>,
        cards_won_by_opponent: CardsSet,
    ) -> PlayTurnResponse {
        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            kwargs
                .set_item("cards", cards.map(|card| Py::new(py, card).unwrap()))
                .unwrap();
            kwargs
                .set_item("board", Py::new(py, Board::new(&fields)).unwrap())
                .unwrap();
            kwargs
                .set_item(
                    "cards_won_by_opponent",
                    Py::new(py, cards_won_by_opponent).unwrap(),
                )
                .unwrap();
            self.bot
                .call_method(py, "play_turn", (), Some(kwargs))
                .expect("Call to play_turn() failed")
                .extract(py)
                .expect("play_turn() returned wrong type")
        })
    }
}

#[pyfunction]
pub fn run_bot(bot: PyObject) {
    PythonBot { bot }.run().unwrap()
}
