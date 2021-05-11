import numpy
from quoridor_python import Game as RustGame

class QuoridorGame:
    def __init__(self):
        self.rust_game = RustGame()

    def add_wall(self, x, y, horizontal=True):
        return self.rust_game.add_wall(x, y, 0 if horizontal else 1)

    def can_add_wall(self, x, y, horizontal=True):
        return self.rust_game.can_add_wall(x, y, 0 if horizontal else 1)

    def can_move_to(self, x, y):
        return self.rust_game.can_move_to((x, y))
    
    def move_to(self, x, y):
        return self.rust_game.move_token_to((x, y))

    def valid_moves_as_numpy(self):
        add_walls_h = [
            self.can_add_wall(x, y, horizontal=True)
            for y in range(9)
            for x in range(9)
        ]
        add_walls_v = [
            self.can_add_wall(x, y, horizontal=False)
            for y in range(9)
            for x in range(9)
        ]
        token_moves = [
            self.can_move_to(x, y)
            for y in range(8)
            for x in range(8)
        ]

        return numpy.array(add_walls_h + add_walls_v + token_moves, dtype="float32")

    def state_as_numpy(self):
        result = numpy.zeros((5, 9, 9), dtype="float32")

        multiplier = 1 if self.rust_game.current_player() == 1 else -1

        x, y = self.rust_game.get_location(1)
        result[0, y, x] = 1 * multiplier
        x, y = self.rust_game.get_location(2)
        result[0, y, x] = -1 * multiplier

        for y in range(9):
            for x in range(9):
                for direction in range(4):
                    is_passible = 1 if self.rust_game.is_passible(x, y, direction) else 0
                    result[direction + 1, y, x] = is_passible

        if self.rust_game.current_player() == 2:
            result = result[:, ::-1, :]

        return result


if __name__ == "__main__":
    g = QuoridorGame()
    g.add_wall(2, 2, horizontal=False)
    print(g.can_add_wall(3, 2, horizontal=False))
    print(g.rust_game.get_location(2))
    print(g.can_move_to(4, 7))
    g.move_to(4, 7)
    print(g.state_as_numpy())
    print(g.valid_moves_as_numpy())