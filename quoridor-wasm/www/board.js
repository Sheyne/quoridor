import * as wasm from "quoridor-wasm";

export class BoardView {
    getInfoForEvent(event, kind, x, y) {
        if (kind == "horizontal") {
            let fraction = (event.pageX - this.div.offsetLeft - event.target.offsetLeft) / event.target.offsetWidth;
            if (fraction < 0.5) {
                x -= 1;
            }
            if (x < 0) { x = 0; }
            if (x > 7) { x = 7; }
        }
        if (kind == "vertical") {
            let fraction = (event.pageY - this.div.offsetTop - event.target.offsetTop) / event.target.offsetHeight;
            if (fraction < 0.5) {
                y -= 1;
            }
            if (y < 0) { y = 0; }
            if (y > 7) { y = 7; }
        }
        return {"kind":  kind, x: x, y: y};
    }

    click(event, info) {
        if (this.onclick) {
            this.onclick(info, event);
        }
    }

    mousemove(event, info) {
        this.focused = info;
        this.render(this.lastGame);
    }
    mouseout(event, info) {
        this.focused = null;
        this.render(this.lastGame);
    }

    constructor(game) {
        this.div = document.createElement("div");
        this.div.classList.add("quoridor-board-wrapper-wrapper");
        let wrapper = document.createElement("div");
        wrapper.classList.add("quoridor-board-wrapper");
        this.div.appendChild(wrapper);
        let div = document.createElement("div");
        div.classList.add("quoridor-board");
        wrapper.appendChild(div);

        div.addEventListener("click", e => {
            if (e.target && e.target.data) {
                let info = this.getInfoForEvent(e, e.target.data.kind, e.target.data.x, e.target.data.y);
                let {kind, x,  y} = info;
                let move = null;
                if (kind == "horizontal") {
                    move = {"AddWall": {location: [x, y], orientation: "Horizontal"}};
                }
                if (kind == "vertical") {
                    move = {"AddWall": {location: [x, y], orientation: "Vertical"}};
                }
                if (kind == "cell") {
                    move = {"MoveTo": [x, y]};
                }
                if (move != null) {
                    if (this.onmove) {
                        this.onmove(move);
                    }
                }
            }
        });
        div.addEventListener("mousemove", e => {
            if (e.target && e.target.data) {
                let info = this.getInfoForEvent(e, e.target.data.kind, e.target.data.x, e.target.data.y);
                this.mousemove(e, info);
            }
        });
        div.addEventListener("mouseout", e => {
            if (e.target && e.target.data) {
                let info = this.getInfoForEvent(e, e.target.data.kind, e.target.data.x, e.target.data.y);
                this.mouseout(e, info);
            }
        });
        
        this.cells = [];
        this.horizontal = [];
        this.vertical = [];
        this.joints = [];

        for (let y = 0; y <= 8; y ++) {
            let rowDiv = document.createElement("div");
            rowDiv.classList.add("quoridor-cell-row");
            rowDiv.classList.add("quoridor-row");
            let floorRowDiv;
            if (y != 8) {
                floorRowDiv = document.createElement("div");
                floorRowDiv.classList.add("quoridor-wall-row");
                floorRowDiv.classList.add("quoridor-row");
            }
            let row = [];
            let wallRow = [];
            let floorWallRow = [];
            let jointsRow = [];
            for (let x = 0; x <= 8; x ++) {
                let cellSpan = document.createElement("div");
                cellSpan.classList.add("quoridor-cell");
                cellSpan.data = {"kind": "cell", x: x, y: y};
                rowDiv.appendChild(cellSpan);
                row.push(cellSpan);
                if (x != 8) {
                    let wall = this.createWall(false);
                    wall.data = {"kind": "vertical", x: x, y: y};
                    rowDiv.appendChild(wall);
                    wallRow.push(wall);
                }
                if (y != 8) {
                    let wall = this.createWall(true);
                    wall.data = {"kind": "horizontal", x: x, y: y};
                    floorRowDiv.appendChild(wall);
                    floorWallRow.push(wall);
                    if (x != 8) {
                        let joint = document.createElement("div");
                        joint.classList.add("quoridor-wall");
                        joint.classList.add("quoridor-joint");
                        joint.data = {"kind": "joint", x: x, y: y};
                        floorRowDiv.appendChild(joint);
                        jointsRow.push(joint);
                    }
                }
            }
            div.appendChild(rowDiv);
            if (y != 8) {
                div.appendChild(floorRowDiv);
            }
            this.cells.push(row);
            this.vertical.push(wallRow);
            this.horizontal.push(floorWallRow);
            this.joints.push(jointsRow);
        }

        this.render(game);
    }

    createWall(horizontal) {
        let span = document.createElement("div");
        span.classList.add("quoridor-wall");
        if (horizontal) {
            span.classList.add("quoridor-horizontal");
        } else {
            span.classList.add("quoridor-vertical");
        }

        return span;
    }

    getCell(x, y) {
        return this.cells[y][x];
    }

    getWall(x, y, horizontal) {
        if (horizontal) {
            return this.horizontal[y][x];
        }
        return this.vertical[y][x];
    }

    getJoint(x, y) {
        return this.joints[y][x];
    }

    render(game) {
        if (this.lastGame && this.lastGame !== game) {
            this.lastGame.free();
            this.lastGame = null;
        }
        this.lastGame = game.copy();

        for (let y = 0; y <= 8; y ++) {
            for (let x = 0; x <= 8; x ++) {
                this.getCell(x, y).classList.remove("closed");
                this.getCell(x, y).classList.remove("hover");
                this.getCell(x, y).classList.remove("player1");
                this.getCell(x, y).classList.remove("player2");
                this.getCell(x, y).classList.remove("arrivable1");
                this.getCell(x, y).classList.remove("arrivable2");
                if (y != 8) {
                    this.getWall(x, y, true).classList.remove("closed");
                    this.getWall(x, y, true).classList.remove("hover");
                }
                if (x != 8) {
                    this.getWall(x, y, false).classList.remove("closed");
                    this.getWall(x, y, false).classList.remove("hover");
                }
                if (y != 8 && x != 8) {
                    this.getJoint(x, y).classList.remove("closed");
                    this.getJoint(x, y).classList.remove("hover");
                }
            }
        }
        
        for (let y = 0; y <= 8; y ++) {
            for (let x = 0; x <= 8; x ++) {
                let gameCopy = game.copy();
                if (gameCopy.apply_move({"MoveTo": [x, y]})) {
                    this.getCell(x, y).classList.add("arrivable" + game.current_player());
                }
                gameCopy.free();
                gameCopy = null;
                if (game.get_wall_status(x,y) == wasm.WallState.Horizontal) {
                    this.getWall(x, y, true).classList.add("closed");
                    this.getJoint(x, y).classList.add("closed");
                    this.getWall(x + 1, y, true).classList.add("closed");
                }
                if (game.get_wall_status(x,y) == wasm.WallState.Vertical) {
                    this.getWall(x, y, false).classList.add("closed");
                    this.getJoint(x, y).classList.add("closed");
                    this.getWall(x, y + 1, false).classList.add("closed");
                }
            }
        }

        if (this.focused) {
            if (this.focused.kind == "horizontal") {
                let x = this.focused.x;
                let y = this.focused.y;
                let gameCopy = game.copy();
                if (gameCopy.apply_move({"AddWall": {location: [x, y], orientation: "Horizontal"}})) {
                    this.getWall(x, y, true).classList.add("hover");
                    this.getJoint(x, y).classList.add("hover");
                    this.getWall(x + 1, y, true).classList.add("hover");
                }
                gameCopy.free();
                gameCopy = null;
            }
            if (this.focused.kind == "vertical") {
                let x = this.focused.x;
                let y = this.focused.y;
                let gameCopy = game.copy();
                if (gameCopy.apply_move({"AddWall": {location: [x, y], orientation: "Vertical"}})) {
                    this.getWall(x, y, false).classList.add("hover");
                    this.getJoint(x, y).classList.add("hover");
                    this.getWall(x, y + 1, false).classList.add("hover");
                }
                gameCopy.free();
                gameCopy = null;
            }
        }

        this.getCell(game.get_location(1).x, game.get_location(1).y).classList.add("player1");
        this.getCell(game.get_location(2).x, game.get_location(2).y).classList.add("player2");
    }
}