import * as wasm from "quoridor-wasm";

let boardDiv = document.createElement("div");
document.body.appendChild(boardDiv);

let rows = [];

for (let y = 0; y <= 8; y ++) {
    let rowDiv = document.createElement("div");
    rowDiv.style.margin = "0";
    rowDiv.style.padding = "0";
    let row = [];
    for (let x = 0; x <= 8; x ++) {
        let cellSpan = document.createElement("span");
        cellSpan.style.borderWidth = "0px";
        cellSpan.style.borderStyle = "solid";
        cellSpan.style.margin = "0";
        cellSpan.style.padding = "0";
        cellSpan.style.display = "inline-block";
        cellSpan.style.width = "2em";
        cellSpan.style.height = "2em";
        if (x != 8) {
            cellSpan.style.borderRightWidth = "2px";
            cellSpan.style.borderRightColor = "#ccc";
        }
        if (y != 8) {
            cellSpan.style.borderBottomWidth = "2px";
            cellSpan.style.borderBottomColor = "#ccc";
        }
        rowDiv.appendChild(cellSpan);
        row.push(cellSpan);
    }
    boardDiv.appendChild(rowDiv);
    rows.push(row);
}

let game = wasm.Game.new();
game.add_wall(7, 3, wasm.Orientation.Horizontal);
game.add_wall(2, 3, wasm.Orientation.Vertical);

function updateUi() {
    for (let y = 0; y <= 8; y ++) {
        for (let x = 0; x <= 8; x ++) {
            let cell = rows[y][x];
            cell.style.backgroundColor = "#eee";
            if (y != 8) {
                cell.style.borderBottomColor = "#ccc";
            }
            if (x != 8) {
                cell.style.borderRightColor = "#ccc";
            }
        }
    }
    
    for (let y = 0; y <= 8; y ++) {
        for (let x = 0; x <= 8; x ++) {
            if (game.get_wall_status(x,y) == wasm.WallState.Horizontal) {
                rows[y][x].style.borderBottomColor = "black";
                rows[y][x+1].style.borderBottomColor = "black";
            }
            if (game.get_wall_status(x,y) == wasm.WallState.Vertical) {
                rows[y][x].style.borderRightColor = "black";
                rows[y + 1][x].style.borderRightColor = "black";
            }
        }
    }
    
    rows[game.get_location(1).y][game.get_location(1).x].style.backgroundColor = "red";
    rows[game.get_location(2).y][game.get_location(2).x].style.backgroundColor = "blue";    
}

updateUi();

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
    updateUi();
});

