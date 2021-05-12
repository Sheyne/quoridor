import numpy
from quoridor_python import Game as RustGame

class QuoridorGame:
    def __init__(self, rust_game=None):
        if rust_game is None:
            self.rust_game = RustGame()
        else:
            self.rust_game = rust_game

    def copy(self):
        return QuoridorGame(self.rust_game.clone())

    def as_str(self):
        return self.rust_game.as_str()

    def get_result(self, player):
        if self.rust_game.get_location(1)[1] == 8:
            return 1 * player
        if self.rust_game.get_location(2)[1] == 0:
            return -1 * player
        if self.rust_game.available_walls(1) == 0 and self.rust_game.available_walls(2) == 0:
            am_closer = self.distance_to_goal(player * -1) > self.distance_to_goal(player)
            return 1 if am_closer else -1
        return 0

    def add_wall(self, x, y, horizontal=True):
        return self.rust_game.add_wall(x, y, 0 if horizontal else 1)

    def can_add_wall(self, x, y, horizontal=True):
        return self.rust_game.can_add_wall(x, y, 0 if horizontal else 1)

    def can_move_to(self, x, y):
        return self.rust_game.can_move_to((x, y))
    
    def move_to(self, x, y):
        return self.rust_game.move_token_to((x, y))

    def execute_move_at_idx(self, move_idx):
        if move_idx < 64 * 2:
            horizontal = move_idx < 64
            if not horizontal:
                move_idx -= 64
            x = move_idx % 8
            y = move_idx // 8
            added = self.add_wall(x, y, horizontal)
        else:
            move_idx -= 64*2
            x = move_idx % 9
            y = move_idx // 9
            added = self.move_to(x, y)

    def valid_moves_as_numpy(self):
        add_walls_h = [
            self.can_add_wall(x, y, horizontal=True)
            for y in range(8)
            for x in range(8)
        ]
        add_walls_v = [
            self.can_add_wall(x, y, horizontal=False)
            for y in range(8)
            for x in range(8)
        ]
        token_moves = [
            self.can_move_to(x, y)
            for y in range(9)
            for x in range(9)
        ]

        return numpy.array(add_walls_h + add_walls_v + token_moves, dtype="float32")

    def canonical_form(self):
        return QuoridorGame(rust_game=self.rust_game.canonical_form())

    def state_as_numpy(self):
        result = numpy.zeros((9, 9, 5), dtype="float32")

        multiplier = self.current_player

        x, y = self.rust_game.get_location(1)
        result[y, x, 0] = 1 * multiplier
        x, y = self.rust_game.get_location(2)
        result[y, x, 0] = -1 * multiplier

        for y in range(9):
            for x in range(9):
                for direction in range(4):
                    is_passible = 1 if self.rust_game.is_passible(x, y, direction) else 0
                    result[y, x, direction + 1] = is_passible

        result = result[:, ::self.current_player, :]

        return result

    def distance_to_goal(self, player):
        return self.rust_game.distance_to_goal(1 if player == 1 else 2)

    def __str__(self):
        h, v, p1l, p2l, p1w, p2w = map(int, self.as_str().split(" "))
        return board_from_numbers(h, v, p1l, p2l, p1w, p2w)

    @property
    def s(self):
        return str(self)

    @property
    def current_player(self):
        return 1 if self.rust_game.current_player() == 1 else -1

def board_from_numbers(h, v, p1l, p2l, p1w, p2w):
    p1l = p1l - 1
    p2l = p2l - 1
    p1l = p1l % 9, p1l // 9
    p2l = p2l % 9, p2l // 9
    g = [[" "] * (9 + 8) for _ in range(9 + 8)]
    g[p1l[1] * 2][p1l[0] * 2] = "1"
    g[p2l[1] * 2][p2l[0] * 2] = "2"

    def is_set(x, y, n):
        return 1 << (y + x * 8) & n != 0

    for y in range(8):
        for x in range(8):
            g[1 + 2*y][2*x] = "-"
            g[1 + 2*y][1 + 2*x] = "+"
            g[1 + 2*y][2 + 2*x] = "-"
            g[2*y][1 + 2*x] = "|"
            g[1 + 2*y][1 + 2*x] = "+"
            g[2 + 2*y][1 + 2*x] = "|"

    for y in range(8):
        for x in range(8):
            if is_set(x, y, h):
                g[1 + 2*y][2*x] = "#"
                g[1 + 2*y][1 + 2*x] = "#"
                g[1 + 2*y][2 + 2*x] = "#"
            if is_set(x, y, v):
                g[2*y][1 + 2*x] = "#"
                g[1 + 2*y][1 + 2*x] = "#"
                g[2 + 2*y][1 + 2*x] = "#"

    return "\n".join("".join(row) for row in g)

if __name__ == "__main__":
    g = QuoridorGame()
    g.add_wall(2, 8, horizontal=True)
    print(g.can_add_wall(2, 8, horizontal=True))
    print(g.rust_game.get_location(2))
    print(g.can_move_to(4, 7))
    g.move_to(4, 7)
    print(g.state_as_numpy())
    print(g.valid_moves_as_numpy())
    print(g.as_str())