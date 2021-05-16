import socket
import json

class TcpQuoridor:

    def __init__(self, server, port):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((server, port))
        self.file = self.sock.makefile()

    def sendraw(self, packet):
        self.sock.sendall((packet + "\n").encode())

    def add_wall(self, x, y, orientation):
        data = json.dumps(
            {"AddWall": {"orientation": orientation, "location": [x, y]}})
        self.sendraw(data)
    
    def move(self, direction):
        data = json.dumps({"MoveToken": direction})
        self.sendraw(data)

    def receive(self):
        return json.loads(next(self.file))

if __name__ == "__main__":
    t = TcpQuoridor("localhost", 3456)
    print(t.receive())
    t.add_wall(2, 4, "Vertical")
    print(t.receive())
    t.move("Up")
    print(t.receive())
