import * as wasm from "quoridor-wasm";

export class BoardView {
    getInfoForEvent(event, kind, x, y) {
        if (kind == "horizontal") {
            let fraction = (event.pageX - event.toElement.offsetLeft) / event.toElement.offsetWidth;
            if (fraction < 0.5) {
                x -= 1;
            }
            if (x < 0) { x = 0; }
            if (x > 7) { x = 7; }
        }
        if (kind == "vertical") {
            let fraction = (event.pageY - event.toElement.offsetTop) / event.toElement.offsetHeight;
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

    init() {
        this.div = document.createElement("div");
        this.div.addEventListener("click", e => {
            if (e.toElement.data) {
                let info = this.getInfoForEvent(e, e.toElement.data.kind, e.toElement.data.x, e.toElement.data.y);
                this.click(e, info);
            }
        });
        this.div.addEventListener("mousemove", e => {
            if (e.toElement.data) {
                let info = this.getInfoForEvent(e, e.toElement.data.kind, e.toElement.data.x, e.toElement.data.y);
                this.mousemove(e, info);
            }
        });
        this.div.addEventListener("mouseout", e => {
            if (e.toElement.data) {
                let info = this.getInfoForEvent(e, e.toElement.data.kind, e.toElement.data.x, e.toElement.data.y);
                this.mouseout(e, info);
            }
        });
        this.cells = [];
        this.horizontal = [];
        this.vertical = [];
        this.joints = [];

        for (let y = 0; y <= 8; y ++) {
            let rowDiv = document.createElement("div");
            rowDiv.style.margin = "0";
            rowDiv.style.padding = "0";
            rowDiv.style.lineHeight = "0";
            rowDiv.style.display = "flex";
            let floorRowDiv = document.createElement("div");
            floorRowDiv.style.margin = "0";
            floorRowDiv.style.padding = "0";
            floorRowDiv.style.lineHeight = "0";
            floorRowDiv.style.display = "flex";
            let row = [];
            let wallRow = [];
            let floorWallRow = [];
            let jointsRow = [];
            for (let x = 0; x <= 8; x ++) {
                let cellSpan = document.createElement("span");
                cellSpan.data = {"kind": "cell", x: x, y: y};
                cellSpan.style.margin = "0";
                cellSpan.style.padding = "0";
                cellSpan.style.display = "flex";
                cellSpan.style.width = "3em";
                cellSpan.style.height = "3em";
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
                        let joint = document.createElement("span");
                        joint.data = {"kind": "joint", x: x, y: y};
                        joint.style.margin = "0";
                        joint.style.padding = "0";
                        joint.style.display = "flex";
                        joint.style.width = ".5em";
                        joint.style.height = ".5em";
                        floorRowDiv.appendChild(joint);
                        jointsRow.push(joint);
                    }
                }
            }
            this.div.appendChild(rowDiv);
            this.div.appendChild(floorRowDiv);
            this.cells.push(row);
            this.vertical.push(wallRow);
            this.horizontal.push(floorWallRow);
            this.joints.push(jointsRow);
        }
    }

    createWall(horizontal) {
        let span = document.createElement("span");
        span.style.borderWidth = "0px";
        span.style.borderStyle = "solid";
        span.style.margin = "0";
        span.style.padding = "0";
        span.style.display = "flex";
        if (horizontal) {
            span.style.height = ".5em";
            span.style.width = "3em";
        } else {
            span.style.width = ".5em";
        }
        span.style.background = "#ccc";

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
        this.lastGame = game;

        for (let y = 0; y <= 8; y ++) {
            for (let x = 0; x <= 8; x ++) {
                this.getCell(x, y).style.backgroundColor = "#eee";
                if (y != 8) {
                    this.getWall(x, y, true).style.backgroundColor = "#ccc";
                }
                if (x != 8) {
                    this.getWall(x, y, false).style.backgroundColor = "#ccc";
                }
                if (y != 8 && x != 8) {
                    this.getJoint(x, y).style.backgroundColor = "#ccc";
                }
            }
        }
        
        for (let y = 0; y <= 8; y ++) {
            for (let x = 0; x <= 8; x ++) {
                if (game.get_wall_status(x,y) == wasm.WallState.Horizontal) {
                    this.getWall(x, y, true).style.backgroundColor = "black";
                    this.getJoint(x, y).style.backgroundColor = "black";
                    this.getWall(x + 1, y, true).style.backgroundColor = "black";
                }
                if (game.get_wall_status(x,y) == wasm.WallState.Vertical) {
                    this.getWall(x, y, false).style.backgroundColor = "black";
                    this.getJoint(x, y).style.backgroundColor = "black";
                    this.getWall(x, y + 1, false).style.backgroundColor = "black";
                }
            }
        }

        if (this.focused) {
            if (this.focused.kind == "horizontal") {
                let x = this.focused.x;
                let y = this.focused.y;
                this.getWall(x, y, true).style.backgroundColor = "#333";
                this.getJoint(x, y).style.backgroundColor = "#333";
                this.getWall(x + 1, y, true).style.backgroundColor = "#333";
            }
            if (this.focused.kind == "vertical") {
                let x = this.focused.x;
                let y = this.focused.y;
                this.getWall(x, y, false).style.backgroundColor = "#333";
                this.getJoint(x, y).style.backgroundColor = "#333";
                this.getWall(x, y + 1, false).style.backgroundColor = "#333";
            }
        }

        this.getCell(game.get_location(1).x, game.get_location(1).y).style.backgroundColor = "red";
        this.getCell(game.get_location(2).x, game.get_location(2).y).style.backgroundColor = "blue";    
    }
}