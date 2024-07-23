from gomori import *
import sys

class SchwarzeneggerBot(Bot):
	def new_game(self, color: Color):
		pass

	def play_first_turn(self, cards: List[Card]) -> Card:
		print("I was elected to lead, not to read. Number 3!", file=sys.stderr)
		return cards[2]

	def play_turn(
		self,
		cards: List[Card],
		board: Board,
		cards_won_by_opponent: CardsSet
	) -> PlayTurnResponse:
		ctp = CardToPlay(card=cards[2], i=0, j=0)
		return PlayTurnResponse([ctp])

run_bot(SchwarzeneggerBot())