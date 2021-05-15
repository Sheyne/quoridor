from Game import Game
from quoridor import *


class QuoridorGame(Game):

    def __init__(self):
        self.n = Quoridor().n

    def getInitBoard(self):
        return Quoridor()

    def getBoardSize(self):
        return len(Quoridor().board.state[0]), len(Quoridor().board.state), 3

    def getActionSize(self):
        player_size = self.n
        wall_size = player_size - 1
        return player_size * player_size + wall_size * wall_size * 2

    def getNextState(self, board: Quoridor, player: int, action):
        a = action
        board = board.copy()
        game_player = board.players[0 if player == 1 else 1]
        player_size = self.n
        wall_size = player_size - 1
        valid = False
        if action < player_size * player_size:
            x = action % player_size
            y = action // player_size
            valid = board.set_player_point(game_player, PlayerLocation(x, y))
            if valid:
                return (board, -player)
        else:
            action = action - player_size * player_size
            if action < wall_size * wall_size:
                # Vertical Wall
                x = action % wall_size
                y = action // wall_size
                valid = board.set_wallpoint(game_player, WallPointLocation(
                    x, y), Orientation.Vertical)
            else:
                # Horizonal Wall
                action = action - wall_size * wall_size
                x = action % wall_size
                y = action // wall_size
                valid = board.set_wallpoint(game_player, WallPointLocation(
                    x, y), Orientation.Horizontal)
            if valid:
                return (board, -player)

        print(board)
        raise Exception(
            f"ILLEGAL MOVE: {player} {x},{y} ACTION: {a}, q: {board.q}")

    def getValidMoves(self, board: Quoridor, player):
        board = board.copy()
        size = board.n
        game_player = board.players[0 if player == 1 else 1]
        player_moves = [0] * size * size
        for x in (-1, 1):
            next_location = game_player.location + (x, 0)
            if board.can_move(game_player, next_location):
                nx = next_location.x
                ny = next_location.y
                player_moves[ny * size + nx] = 1
        for y in (-1, 1):
            next_location = game_player.location + (0, y)
            if board.can_move(game_player, next_location):
                nx = next_location.x
                ny = next_location.y
                player_moves[ny * size + nx] = 1
        wall_size = size - 1
        wall_moves = [0] * 2 * wall_size * wall_size
        index = 0
        for orientation in [Orientation.Vertical, Orientation.Horizontal]:
            for y in range(wall_size):
                for x in range(wall_size):
                    if board.try_wallpoint(
                            game_player, WallPointLocation(x, y), orientation):
                        wall_moves[index] = 1
                    index += 1
        return player_moves + wall_moves

    def getGameEnded(self, board: Quoridor, player: int):
        if board.players[0].location.y == board.players[0].finish:
            return player
        elif board.players[1].location.y == board.players[1].finish:
            return -player
        elif all(player.walls == 0 for player in board.players):
            p1 = len(board.find_path(board.players[0]))
            p2 = len(board.find_path(board.players[1]))
            return player if p1 <= p2 else -player
        else:
            return 0

    def getCanonicalForm(self, board: Quoridor, player):
        board = board.copy()
        if player == 1:
            return board
        else:
            # state = board.board.state
            # state.reverse()
            # for row in state:
            #     row.reverse()
            board.players.reverse()
            board.q = not board.q
            return board

    def getSymmetries(self, board, pi):
        return []

    def stringRepresentation(self, board: Quoridor):
        return board.to_string()


if __name__ == "__main__":
    g = QuoridorGame()
    b = g.getInitBoard()
