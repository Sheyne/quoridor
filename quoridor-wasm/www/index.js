import * as wasm from "quoridor-wasm";
import { BoardView } from "./board";

let view = new BoardView();
view.init();
document.body.appendChild(view.div);

let game = wasm.Game.new();

view.onclick = (info, event) => {
    let {kind, x,  y} = info;
    if (kind == "horizontal") {
        game.add_wall(x, y, wasm.Orientation.Horizontal);
    }
    if (kind == "vertical") {
        game.add_wall(x, y, wasm.Orientation.Vertical);
    }
    view.render(game);
};

view.render(game);

window.addEventListener("keyup", function (e) {
    if (e.keyCode == '38') {
        game.move_token(wasm.Direction.Up);
    }
    else if (e.keyCode == '40') {
        game.move_token(wasm.Direction.Down);
    }
    else if (e.keyCode == '37') {
        game.move_token(wasm.Direction.Left);
    }
    else if (e.keyCode == '39') {
        game.move_token(wasm.Direction.Right);
    }
    view.render(game);
});

