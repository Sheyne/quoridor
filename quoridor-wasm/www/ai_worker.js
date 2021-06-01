import * as wasm from "quoridor-wasm";

let ai = new wasm.Ai();

onmessage = function(e) {
    if (e.data.move) {
        if (e.data.move.Restart !== undefined) {
            ai.free();
            ai = new wasm.Ai();
        } else if (e.data.move.StartAt) {
            ai.free();
            ai = new wasm.Ai();
            for (let move of e.data.move.StartAt) {
                ai.send(move);
            }
        } else {
            ai.send(e.data.move);
            postMessage(ai.receive());
        }
    } else if (e.data.setMode == "greedy") {
        ai.set_greedy()
    } else if (e.data.setMode && e.data.setMode.rubot) {
        ai.set_rubot(e.data.setMode.rubot)
    }
}
  