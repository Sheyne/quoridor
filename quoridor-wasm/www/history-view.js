export class HistoryView {
    constructor() {
        this.div = document.createElement("div");
        this.div.classList.add("history");
        this.states = [];
        this.selectedBoard = -1;

        document.addEventListener("keyup", e => {
            if (e.code == "ArrowUp" || e.code == "ArrowLeft") {
                this.selectBoard(this.selectedBoard - 1);
            }
            else if (e.code == "ArrowDown" || e.code == "ArrowRight") {
                this.selectBoard(this.selectedBoard + 1);
            }
        }); 
    }

    selectedGame() {
        return this.states[this.selectedBoard].game.copy();
    }
    
    movesToSelected() {
        let moves = [];
        for (let i = 1; i <= this.selectedBoard; i ++) {
            moves.push(this.states[i].move);
        }
        return moves;
    }

    clearPastSelected() {
        for (let deleted of this.states.splice(this.selectedBoard + 1)) {
            deleted.game.free();
            this.div.removeChild(deleted);
        }
    }

    isAtPresent() {
        return this.selectedBoard + 1 == this.states.length;
    }

    selectBoard(index) {
        if (index >= this.states.length || index < 0) {
            return;
        }
        if (this.selectedBoard >= 0) {
            this.states[this.selectedBoard].classList.remove("selected");
        }
        this.selectedBoard = index;
        this.states[this.selectedBoard].classList.add("selected");

        if (this.onselect) {
            this.onselect(this.states[this.selectedBoard].game, this.states[this.selectedBoard].index);
        }
    }

    addBoard(game, move) {
        let view = document.createElement("div");
        view.classList.add("history-entry");

        let message = "???";
        if (move.MoveTo) {
            message = "Move to " + move.MoveTo[0] + ", " + move.MoveTo[1];
        } else if (move.InitialSetUp === null) {
            message = "Setup";
        } else if (move.AddWall) {
            message = "Add " + move.AddWall.orientation.toLowerCase() + " wall at " + move.AddWall.location[0] + ", " + move.AddWall.location[1];
        }

        view.textContent = this.states.length + ": " + message;
        view.index = this.states.length;
        view.game = game;
        view.move = move;
        view.addEventListener("click", () => this.selectBoard(view.index));

        this.div.appendChild(view);

        this.states.push(view);

        if (this.selectedBoard + 2 == this.states.length) {
            this.selectBoard(this.selectedBoard + 1);
        }
    }
}