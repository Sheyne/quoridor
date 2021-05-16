from MCTS import MCTS
from nnet import NNetWrapper as nn
from alpha_zero_game import AlphaZeroQuoridorGame as Game
from game_wrapper import MoveTo, AddWall
import numpy as np
from tcp_quoridor import TcpQuoridor


class dotdict(dict):
    def __getattr__(self, name):
        return self[name]

def send_move(q, board, move):
    if isinstance(move, AddWall):
        x, y, horizontal = move
        q.add_wall(int(x), int(y), "Horizontal" if horizontal else "Vertical")
    if isinstance(move, MoveTo):
        x, y = move
        mx, my = board.player_location(board.current_player)
        if x == mx and y + 1 == my:
            q.move("Up")
        elif x == mx and y - 1 == my:
            q.move("Down")
        elif y == my and x + 1 == mx:
            q.move("Left")
        elif y == my and x - 1 == mx:
            q.move("Right")
        else:
            raise(Exception("I'm confused"))

tcp = TcpQuoridor("localhost", 5611)
g = Game()
n1 = nn(g)
n1.load_checkpoint('./temp/', 'best.h5')
args1 = dotdict({'numMCTSSims': 50, 'cpuct':1.0})
mcts1 = MCTS(g, n1, args1)
n1p = lambda x: np.argmax(mcts1.getActionProb(x, temp=0))
board = g.getInitBoard()

while True:
    idx = n1p(board)
    send_move(tcp, board, board.translate_idx(idx))
    board.execute_move_at_idx(idx)
    move = tcp.receive()
    if "AddWall" in move:
        move = move["AddWall"]
        board.add_wall(move["location"][0], move["location"][1], move["orientation"] == "Horizontal")
    elif "MoveToken" in move:
        move = move["MoveToken"]
        dir = {"Up": 0, "Down": 1, "Left": 2, "Right": 3}
        move = dir[move]
        board.move(move)
    else:
        raise(Exception("I'm confused"))
