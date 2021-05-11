from game_wrapper import QuoridorGame

class AlphaZeroQuoridorGame:
    def check_player_valid(self, board, player):
        assert board.current_player == player

    def getInitBoard(self):
        return QuoridorGame()

    def getBoardSize(self):
        return (9, 9)

    def getActionSize(self):
        return (9*9 + 8*8*2)

    def getNextState(self, board, player, action):
        self.check_player_valid(board, player)
        board = board.copy()
        board.execute_move_at_idx(action)
        return (board, board.current_player)

    def getValidMoves(self, board, player):
        self.check_player_valid(board, player)
        return board.valid_moves_as_numpy()

    def getGameEnded(self, board, player):
        return board.get_result(player)

    def getCanonicalForm(self, board, player):
        return board.canonical_form()

    def getSymmetries(self, board, pi):
        return []

    def stringRepresentation(self, board):
        return board.as_str()

    def getScore(self, board, player):
        return board.distance_to_goal(-1 * player) - board.distance_to_goal(player)
