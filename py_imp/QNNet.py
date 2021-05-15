from keras.optimizers import *
from keras.layers import *
from keras.models import *
import argparse
from utils import *
import sys
sys.path.append('..')


class QNNet():
    def __init__(self, game, args):
        # game params
        self.board_x, self.board_y, self.board_dims = game.getBoardSize()
        self.action_size = game.getActionSize()
        self.args = args

        # Neural Net
        # s: batch_size x board_x x board_y
        self.input_boards = Input(
            shape=(self.board_x, self.board_y, self.board_dims))

        h_conv1 = Activation('relu')(BatchNormalization(axis=3)(Conv2D(args.num_channels, 3, padding='same')(
            self.input_boards)))         # batch_size  x board_x x board_y x num_channels
        h_conv2 = Activation('relu')(BatchNormalization(axis=3)(Conv2D(args.num_channels, 3, padding='same')(
            h_conv1)))         # batch_size  x board_x x board_y x num_channels
        h_conv3 = Activation('relu')(BatchNormalization(axis=3)(Conv2D(args.num_channels, 3, padding='same')(
            h_conv2)))        # batch_size  x (board_x) x (board_y) x num_channels
        h_conv4 = Activation('relu')(BatchNormalization(axis=3)(Conv2D(args.num_channels, 3, padding='valid')(
            h_conv3)))        # batch_size  x (board_x-2) x (board_y-2) x num_channels
        h_conv4_flat = Flatten()(h_conv4)
        s_fc1 = Dropout(args.dropout)(Activation('relu')(BatchNormalization(
            axis=1)(Dense(1024)(h_conv4_flat))))  # batch_size x 1024
        s_fc2 = Dropout(args.dropout)(Activation('relu')(BatchNormalization(
            axis=1)(Dense(512)(s_fc1))))          # batch_size x 1024
        self.pi = Dense(self.action_size, activation='softmax', name='pi')(
            s_fc2)   # batch_size x self.action_size
        self.v = Dense(1, activation='tanh', name='v')(
            s_fc2)                    # batch_size x 1

        self.model = Model(inputs=self.input_boards, outputs=[self.pi, self.v])
        self.model.compile(
            loss=['categorical_crossentropy', 'mean_squared_error'], optimizer=Adam(args.lr))
