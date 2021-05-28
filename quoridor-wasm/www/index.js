import * as wasm from "quoridor-wasm";
import { BoardView } from "./board";

let view = new BoardView();
view.init();
document.body.appendChild(view.div);

let game = new wasm.Game();
let ai = new wasm.Ai("greedy");

view.onclick = (info, event) => {
    let {kind, x,  y} = info;
    let move = null;
    if (kind == "horizontal") {
        move = wasm.Move.add_wall(x, y, wasm.Orientation.Horizontal);
    }
    if (kind == "vertical") {
        move = wasm.Move.add_wall(x, y, wasm.Orientation.Vertical);
    }
    if (move != null) {
        if (game.apply_move(move)) {
            ai.send(move);
            view.render(game);
            game.apply_move(ai.receive());
        }
    }
    view.render(game);
};

view.render(game);

window.addEventListener("keyup", function (e) {
    let move = null;
    if (e.keyCode == '38') {
        move = wasm.Move.move_token(wasm.Direction.Up);
    }
    else if (e.keyCode == '40') {
        move = wasm.Move.move_token(wasm.Direction.Down);
    }
    else if (e.keyCode == '37') {
        move = wasm.Move.move_token(wasm.Direction.Left);
    }
    else if (e.keyCode == '39') {
        move = wasm.Move.move_token(wasm.Direction.Right);
    }
    if (move != null) {
        if (game.apply_move(move)) {
            ai.send(move);
            view.render(game);
            game.apply_move(ai.receive());
        }
    }
    view.render(game);
});

