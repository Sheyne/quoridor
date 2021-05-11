from game_wrapper import QuoridorGame
import numpy

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

    def prettyStringRepresentation(self, board):
        return str(board)

    def getScores(self, board, player):
        scores = numpy.zeros(self.getActionSize())
        for idx, valid in enumerate(self.getValidMoves(board, player)):
            if valid:
                next_state, next_player = self.getNextState(board, player, idx)
                scores[idx] = self.getScore(next_state, player)

        min_score = scores.min()
        if min_score > 0:
            min_score = 0
        adjusted_scores = (scores - min_score)
        masked_scores = adjusted_scores * self.getValidMoves(board, player)
        return  masked_scores, scores.mean()


    def getScore(self, board, player):
        return board.distance_to_goal(-1 * player) - board.distance_to_goal(player)


if __name__ == "__main__":
    game = AlphaZeroQuoridorGame()
    board = game.getInitBoard()
    player = 1
    for _ in range(1000):
        print("--------")
        scores, e = game.getScores(board, player)
        action = scores.argmax()
        print(action)
        board, player = game.getNextState(board, player, action)
        print(game.prettyStringRepresentation(board))
        ended = game.getGameEnded(board, player * -1)
        if ended != 0:
            print(f"Player {player * -1} {'wins' if ended == 1 else 'loses'}")
            break

