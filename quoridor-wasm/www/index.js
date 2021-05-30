import * as wasm from "quoridor-wasm";
import { BoardView } from "./board";
import Worker from "./bootstrap.worker.js";
import { getConnection } from "./rtc";

let playAi = document.createElement("button");
playAi.textContent = "Play Ai";
document.body.appendChild(playAi);
let playFriend = document.createElement("button");
playFriend.textContent = "Play Friend";
document.body.appendChild(playFriend);

playFriend.onclick = async () => {
    document.body.removeChild(playFriend);
    document.body.removeChild(playAi);

    let connection = await getConnection();
    my_turn = connection.kind == "serve";
    connection.onmessage = e => opponent.onmessage(JSON.parse(e.data));
    opponent.postMessage = m => connection.send(JSON.stringify(m));
    opponent.onstart();
};
playAi.onclick = async () => {
    document.body.removeChild(playFriend);
    document.body.removeChild(playAi);

    let slider = document.createElement("input");
    let sliderSpan = document.createElement("label");
    sliderSpan.textContent = "Difficulty: ";
    document.body.appendChild(sliderSpan);
    slider.type = "range";
    slider.min = "0";
    slider.max = "4000";
    slider.value = "1500";
    slider.addEventListener("change", (e) =>  ai_worker.postMessage({setMode: {rubot: Number(slider.value)}}));
    document.body.appendChild(slider);

    let ai_worker = new Worker();
    ai_worker.onmessage = e => opponent.onmessage(e.data);
    opponent.postMessage = m => ai_worker.postMessage({move: m});
    opponent.onstart();
};

function startgame() {
    let view = new BoardView();
    view.init();
    document.body.appendChild(view.div);
    
    opponent.onmessage = data => {
        game.apply_move(data);
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
                opponent.postMessage(move);
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
        if (e.code == "ArrowUp") {
            move = {"MoveToken": "Up"};
        }
        else if (e.code == "ArrowDown") {
            move = {"MoveToken": "Down"};
        }
        else if (e.code == "ArrowLeft") {
            move = {"MoveToken": "Left"};
        }
        else if (e.code == "ArrowRight") {
            move = {"MoveToken": "Right"};
        }
        if (move != null) {
            if (game.apply_move(move)) {
                my_turn = false;
                opponent.postMessage(move);
                view.render(game);
            }
        }
        view.render(game);
    });

}

let game = new wasm.Game();
let my_turn = true;
let opponent = {"onstart": startgame};