from enum import Enum
from dataclasses import dataclass
from typing import List, Optional, Dict
import heapq
import numpy


@dataclass(unsafe_hash=True, order=True)
class Location:
    x: int
    y: int

    def value(self):
        return (self.x, self.y)

    def to_base(self):
        return Location(*self.value())

    def __add__(self, delta):
        (x, y) = delta
        return type(self)(self.x + x, self.y + y)


class PlayerLocation(Location):
    def value(self):
        return (self.x*2, self.y*2)


class WallPointLocation(Location):
    def value(self):
        return (self.x*2 + 1, self.y*2 + 1)


class PointState(Enum):
    EMPTY = 1
    PLAYER = 2
    WALL = 3
    WALLPOINTV = 4
    WALLPOINTH = 5
    WALLPOINTEMPTY = 6


class Orientation(Enum):
    Horizontal = 1
    Vertical = 2


class FinishLine(Enum):
    NegX = 1
    PosX = 2
    NegY = 3
    PosY = 4


@dataclass
class Player:
    location: PlayerLocation
    finish: int
    walls: int

    def value(self):
        return self.location.to_base(), self.finish, self.walls

    def copy(self):
        return Player(self.location, self.finish, self.walls)


class Board:
    state: List[List[PointState]] = []

    def __init__(self, state: List[List[PointState]]):
        self.n = len(state)
        self.state = state

    def to_grid(self):
        d = {
            PointState.EMPTY: 1,
            PointState.PLAYER: 1,
            PointState.WALL: 0,
            PointState.WALLPOINTV: 0,
            PointState.WALLPOINTH: 0,
            PointState.WALLPOINTEMPTY: 0,
        }

        return [[1] * self.n] + [([d[point] for point in arr]) for arr in self.state] + [[1] * self.n]

    def get_point(self, location: Location) -> Optional[PointState]:
        (x, y) = location.value()
        return self.state[y][x] if 0 <= x < self.n and 0 <= y < self.n else None

    def set_point(self, location: Location, point_state: PointState):
        (x, y) = location.value()
        self.state[y][x] = point_state

    def set_wallpoint(self, location: Location, orientation: Orientation):
        (x, y) = location.value()
        if orientation is Orientation.Horizontal:
            self.set_point(location.to_base(), PointState.WALLPOINTH)
            for n in [-1, 1]:
                self.set_point(Location(x+n, y), PointState.WALL)
            return True

        elif orientation is Orientation.Vertical:
            self.set_point(location.to_base(), PointState.WALLPOINTV)
            for n in [-1, 1]:
                self.set_point(Location(x, y+n), PointState.WALL)
            return True
        return False

    def can_move(self, l1, l2):
        x1, y1 = l1.value()
        x2, y2 = l2.value()

        if x1 == x2 and abs(y1 - y2) == 2:
            # Vertical Movement
            middle_y = min(y1, y2) + 1
            return self.get_point(Location(x1, middle_y)) is PointState.EMPTY
        elif y1 == y2 and abs(x1 - x2) == 2:
            # Horizontal Movement
            middle_x = min(x1, x2) + 1
            return self.get_point(Location(middle_x, y1)) is PointState.EMPTY
        else:
            return False

    def can_set_wall(self, location: Location):
        x, y = location.value()
        return (x+y) % 2 == 1 and self.get_point(location.to_base()) is PointState.EMPTY

    def try_wallpoint(self, player: Player, location: WallPointLocation, orientation: Orientation, allPlayers: list[Player]) -> bool:
        if player.walls > 0 and self.get_point(location.to_base()) is PointState.WALLPOINTEMPTY:
            if orientation is Orientation.Horizontal:
                return all(self.can_set_wall(location.to_base() + (n, 0)) for n in [-1, 1]) and self.try_with_wall(allPlayers, location, orientation)
            elif orientation is Orientation.Vertical:
                return all(self.can_set_wall(location.to_base() + (0, n)) for n in [-1, 1]) and self.try_with_wall(allPlayers, location, orientation)
        return False

    def try_with_wall(self, players: list[Player], location: Location, orientation: Orientation) -> bool:
        try_board = self.copy()
        try_board.set_wallpoint(location, orientation)
        return all(try_board.has_path(p) for p in players)

    def find_path(self, player: Player):
        return list(self.a_star_search(player.location.to_base(), player.finish))

    def has_path(self, player):
        return len(self.find_path(player)) > 0

    def a_star_search(self, start: Location, goal: int):
        def heuristic(y1: int, y2: int) -> float:
            return abs(y1 - y2)

        def neighbors(location):
            directions = [
                (2, 0),
                (-2, 0),
                (0, 2),
                (0, -2)
            ]

            return [l for l in (location + direction for direction in directions) if self.can_move(location, l)]

        frontier: list[Location] = []
        heapq.heappush(frontier, (0, start))
        came_from: Dict[Location, Optional[Location]] = {}
        cost_so_far: Dict[Location, float] = {}
        came_from[start] = None
        cost_so_far[start] = 0

        while frontier:
            _, current = heapq.heappop(frontier)

            if current.y == goal:
                break

            for next in neighbors(current):
                new_cost = cost_so_far[current] + 1
                if next not in cost_so_far or new_cost < cost_so_far[next]:
                    cost_so_far[next] = new_cost
                    priority = new_cost + heuristic(next.y, goal)
                    heapq.heappush(frontier, (priority, next))
                    came_from[next] = current

        def recover_path():
            p = current
            while p:
                yield p
                p = came_from[p]

        return reversed(list(recover_path()))

    def copy(self):
        return Board([[point for point in row] for row in self.state])


@ dataclass
class Quoridor:
    players = []

    def get_initial_board(self):
        return [
            ([
                PointState.EMPTY,
                PointState.WALLPOINTEMPTY if row % 2 == 1 else PointState.EMPTY
            ] * (self.n-1)
                + [PointState.EMPTY])
            for row in range(self.n*2 - 1)
        ]

    def get_initial_players(self, n=9, players_count=2, walls_count=10):
        players = []
        starting_points: List[(PlayerLocation, PlayerLocation)] = [
            (PlayerLocation(int(n/2), 0), n*2-1),
            (PlayerLocation(int(n/2), n-1), 0),
        ]
        if (players_count > len(starting_points)):
            raise ValueError

        for i in range(min(players_count, len(starting_points))):
            start, finish = starting_points[i]
            player = Player(location=start, finish=finish, walls=walls_count)
            players.append(player)
            self.set_point(start, PointState.PLAYER)

        return players

    def __init__(self, board: Optional[Board] = None, players: Optional[list[Player]] = None, n=9, player_count=2, walls_count=10, q=False):
        self.q = q
        self.n = n
        self.board: Board = board or Board(self.get_initial_board())
        self.players: list[Player] = players or self.get_initial_players(
            n, player_count, walls_count)

    def get_point(self, location: Location) -> PointState or None:
        return self.board.get_point(location)

    def set_point(self, location: Location, point_state: PointState):
        self.board.set_point(location, point_state)

    def set_player_point(self, player: Player, location: PlayerLocation, force: bool = False) -> bool:
        if force or self.can_move(player, location):
            prev_location = player.location
            player.location = location
            self.set_point(location.to_base(), PointState.PLAYER)
            if player.location.value() != prev_location.value() and all(p.location.value() != prev_location.value() for p in self.players):
                self.set_point(prev_location.to_base(), PointState.EMPTY)
            return True
        else:
            return False

    def can_move(self, player: Player, location: Location) -> bool:
        return self.board.can_move(player.location, location)

    def to_grid(self):
        return self.board.to_grid()

    def set_wallpoint(self, player: Player, location: WallPointLocation, orientation: Orientation):
        if self.try_wallpoint(player, location, orientation):
            self.board.set_wallpoint(location, orientation)
            player.walls -= 1
            return True

        return False

    def can_set_wall(self, location: Location):
        return self.board.can_set_wall(location)

    def try_wallpoint(self, player: Player, location: WallPointLocation, orientation: Orientation) -> bool:
        return self.board.try_wallpoint(player, location, orientation, self.players)

    def try_with_wall(self, location: Location, orientation: Orientation) -> bool:
        return self.board.try_with_wall(self.players, location, orientation)

    def find_path(self, player: Player):
        return self.board.find_path(player)

    def get_next_step(self, player: Player):
        path = q.find_path(player)
        return path[1]

    def has_path(self, player: Player):
        return len(self.find_path(player)) > 0

    def to_string(self) -> str:
        d = {
            PointState.WALLPOINTV: "|",
            PointState.WALLPOINTH: "-",
            PointState.WALLPOINTEMPTY: ""
        }

        wall_string = ""
        for x in range(self.n-1):
            for y in range(self.n-1):
                point_string = d[self.get_point(WallPointLocation(x, y))]
                if point_string:
                    wall_string = wall_string + point_string + f"{x},{y}"
        players_string = "".join(
            f"p{player.location.x},{player.location.y}{player.finish}/{player.walls}" for player in self.players)

        return wall_string + players_string

    def to_numpy(self):
        b = numpy.array(self.board.state)
        walls = (b == PointState.WALL) | (
            b == PointState.WALLPOINTH) | (b == PointState.WALLPOINTV)
        player1 = numpy.zeros_like(walls)
        player2 = numpy.zeros_like(walls)
        x1, y1 = self.players[0].location.value()
        player1[y1, x1] = 1
        x2, y2 = self.players[1].location.value()
        player2[y2, x2] = 1
        result = numpy.array([walls, player1, player2]).transpose(1, 2, 0)
        if self.q:
            return result[:, ::-1, ::-1]
        return result

    def copy(self):
        return Quoridor(self.board.copy(), [p.copy() for p in self.players])

    def __str__(self):
        d = {
            PointState.EMPTY: " ",
            PointState.PLAYER: "X",
            PointState.WALLPOINTEMPTY: "@",
            PointState.WALLPOINTV: "|",
            PointState.WALLPOINTH: "-",
            PointState.WALL: "#",
        }

        return "\n".join("".join(d[x] for x in row) for row in self.board.state)


if __name__ == "__main__":
    q = Quoridor()
    p1, p2 = q.players
    q.set_wallpoint(p1, WallPointLocation(5, 5), Orientation.Horizontal)
    print(q.to_string())
    # from tcp_quoridor import TcpQuoridor
    # t = TcpQuoridor("localhost", 3456)
    # q = Quoridor()

    # deltas = {
    #     "Up": (0, -1),
    #     "Down": (0, 1),
    #     "Left": (-1, 0),
    #     "Right": (1, 0)
    # }
    # while True:
    #     move = t.receive()
    #     if "MoveToken" in move:
    #         print(q)
    #         delta = deltas[move["MoveToken"]]
    #         q.set_player_point(
    #             q.players[0], q.players[0].location + delta)

    #     if "AddWall" in move:
    #         o = {
    #             "Horizontal": Orientation.Horizontal,
    #             "Vertical": Orientation.Vertical
    #         }
    #         x, y = move["AddWall"]["location"]
    #         orientation = o[move["AddWall"]["orientation"]]
    #         q.set_wallpoint(q.players[0], WallPointLocation(x, y), orientation)
    #     print(q)

    #     next_move = q.get_next_step(q.players[1])
    #     for direction, delta in deltas.items():
    #         if (q.players[1].location + delta).value() == next_move.value():
    #             t.move(direction)
    #             break
    #     else:
    #         print(f"No move matched, next_move: {next_move}")
    #     q.set_player_point(q.players[1], PlayerLocation(
    #         next_move.x//2, next_move.y//2))
