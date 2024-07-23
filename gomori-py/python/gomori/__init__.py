from gomori._gomori import *
from typing import List

class Bot():
	def new_game(self, color: Color):
		raise NotImplementedError()

	def play_first_turn(cards: List[Card]) -> Card:
		raise NotImplementedError()

	def play_turn(
		cards: List[Card],
		board: Board,
		cards_won_by_opponent: CardsSet
	) -> PlayTurnResponse:
		raise NotImplementedError()
