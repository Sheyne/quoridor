import * as wasm from "quoridor-wasm";
import { BoardView } from "./board";
import Worker from "./bootstrap.worker.js";

let view = new BoardView();
view.init();
document.body.appendChild(view.div);

let game = new wasm.Game();
let my_turn = true;

let ai_worker = new Worker();
ai_worker.onmessage = function (event) {
    game.apply_move(event.data);
    view.render(game);
    my_turn = true;
};

view.onclick = (info, event) => {
    if (!my_turn)
        return;
    let {kind, x,  y} = info;
    let move = null;
    if (kind == "horizontal") {
        move = {"AddWall": {location: [x, y], orientation: "Horizontal"}};
    }
    if (kind == "vertical") {
        move = {"AddWall": {location: [x, y], orientation: "Vertical"}};
    }
    if (move != null) {
        if (game.apply_move(move)) {
            my_turn = false;
            ai_worker.postMessage(move);
            view.render(game);
        }
    }
    view.render(game);
};

view.render(game);

window.addEventListener("keyup", function (e) {
    if (!my_turn)
        return;
    let move = null;
    if (e.keyCode == '38') {
        move = {"MoveToken": "Up"};
    }
    else if (e.keyCode == '40') {
        move = {"MoveToken": "Down"};
    }
    else if (e.keyCode == '37') {
        move = {"MoveToken": "Left"};
    }
    else if (e.keyCode == '39') {
        move = {"MoveToken": "Right"};
    }
    if (move != null) {
        if (game.apply_move(move)) {
            my_turn = false;
            ai_worker.postMessage(move);
            view.render(game);
        }
    }
    view.render(game);
});

