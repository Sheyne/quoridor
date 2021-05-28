import * as wasm from "quoridor-wasm";

let ai = new wasm.Ai();
ai.set_greedy()

onmessage = function(e) {
    if (e.data.move) {
        ai.send(e.data.move);
        postMessage(ai.receive());
    } else if (e.data.setMode == "greedy") {
        ai.set_greedy()
    } else if (e.data.setMode && e.data.setMode.rubot) {
        // ai.set_rubot(e.data.setMode.rubot)
    }
}
  