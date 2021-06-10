import * as wasm from "quoridor-wasm";
import { BoardView } from "./board";
import { HistoryView } from "./history-view";
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
    myTurn = connection.kind == "serve";
    firstPlayer = myTurn;
    connection.onmessage = e => opponent.onmessage(JSON.parse(e.data));
    opponent.postMessage = m => connection.send(JSON.stringify(m));
    opponent.onstart();
};
playAi.onclick = async () => {
    aiOpponent = true;
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
    let result = null;
    let game = new wasm.Game();
    let view = new BoardView(game);
    let history = new HistoryView();
    history.addBoard(game.copy(), {"InitialSetUp": null});

    let playArea = document.createElement("div");
    playArea.classList.add("play-area");
    document.body.appendChild(playArea);
    playArea.appendChild(view.div);
    playArea.appendChild(history.div);
    let infoDiv = document.createElement("div");
    let playAgain = document.createElement("button");
    playAgain.textContent = "Play Again";
    document.body.appendChild(infoDiv);
    document.body.appendChild(playAgain);

    history.onselect = (game, _index) => {
        view.render(game);
    };

    let restart = () => {
        game = new wasm.Game();
        result = null;
        myTurn = firstPlayer;
        history.selectBoard(0);
        history.clearPastSelected();
        // view.render(game);
    };

    playAgain.addEventListener("click", () => {
        opponent.postMessage({"Restart": !myTurn});
        restart();
    });

    let updateResult = () => {
        if (result = game.result()) {
            infoDiv.innerHTML = "Player " + result + " wins";
        }else {
            infoDiv.innerHTML = "Player 1 has " + game.available_walls(1) + " walls left.<br/>" +
                                    "Player 2 has " + game.available_walls(2) + " walls left.<br/>";
        }
    };
    updateResult();

    opponent.onmessage = data => {
        if (data.Restart !== undefined) {
            restart();
        } else {
            if (myTurn || result) {
                return;
            }
            game.apply_move(data);
            history.addBoard(game.copy(), data);

            updateResult();
            myTurn = true;
        }
    };

    view.onmove = (move) => {
        if (!history.isAtPresent()) {
            if (aiOpponent) {
                history.clearPastSelected();
                let movesToSelected = history.movesToSelected();
                opponent.postMessage({"StartAt": movesToSelected});
                game.free();
                game = history.selectedGame();
            } else {
                if (!myTurn || result)
                    return;
            }
        } else {
            if (!myTurn || result)
                return;
        }
        
        if (game.apply_move(move)) {
            history.addBoard(game.copy(), move);
            myTurn = false;
            opponent.postMessage(move);

            updateResult();
        }
    };
}

let firstPlayer = true;
let myTurn = true;
let aiOpponent = false;
let opponent = {"onstart": startgame};