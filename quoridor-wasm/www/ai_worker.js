import * as wasm from "quoridor-wasm";

let ai = new wasm.Ai("rubot");

onmessage = function(e) {
    ai.send(e.data);
    postMessage(ai.receive());
}
  