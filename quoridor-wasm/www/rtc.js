
class RTCCommunication {
    gotOnMessage(e) {
        if (this.onmessage) {
            this.onmessage(e);
        }
    }
    gotOnStateChange(e) {
        if (this.onstatechange) {
            this.onstatechange(e);
        }
    }
    gotDataChannel(e) {
        console.log("gotDataChannel", e);
        this.channel = e.channel;
        this.channel.onopen = e => this.gotChannelOpen(e);
        this.channel.onmessage = e => this.gotOnMessage(e);
        if (this.ondatachannel) {
            this.ondatachannel(e);
        }
    }
    gotChannelOpen(e) {
        if (this.onopen) {
            this.onopen(e);
        }
    }

    send(message) {
        this.channel.send(message);
    }

    constructor() {
        this.connection = new RTCPeerConnection({ 
            iceTransportPolicy: "all",
            iceCandidatePoolSize: "0",
            iceServers: [{
                    urls: [
                        "stun:stun.l.google.com:19302",
                        "stun:stun1.l.google.com:19302",
                        "stun:stun2.l.google.com:19302",
                        "stun:stun3.l.google.com:19302",
                        "stun:stun4.l.google.com:19302",
                    ]
                }]
        });
        this.connection.onicecandidate = e => this.gotIceCaandidate(e);
        this.connection.onsignalingstatechange = e => this.gotOnStateChange(e);
        this.connection.onconnectionstatechange = e => this.gotOnStateChange(e);
        this.icePromise = new Promise((resolve, reject) => {
            this.connection.onicecandidate = e => {
                if (e.candidate == null) {
                    resolve();
                }
            };
        })
    }

    async serve() {
        this.kind = "serve";
        this.gotDataChannel({channel: this.connection.createDataChannel('channel')});
        let offer = await this.connection.createOffer();
        this.connection.setLocalDescription(offer);
        await this.icePromise;

        return this.connection.localDescription;
    }

    async connect(offer) {
        this.kind = "client";
        this.connection.ondatachannel = e => this.gotDataChannel(e);

        this.connection.setRemoteDescription(offer);
        let answer = await this.connection.createAnswer();
        this.connection.setLocalDescription(answer);
        await this.icePromise;
        return this.connection.localDescription;
    }
}

export function getConnection() {
    return new Promise(getConnectionInner);
}

function getConnectionInner(resolve, reject) {
    try {
        let inputBox = document.createElement("textarea");
        let button = document.createElement("button");
        button.textContent = "Go";
        document.body.appendChild(inputBox);
        document.body.appendChild(button);

        let communication = new RTCCommunication();
        communication.onopen = e => {
            document.body.removeChild(inputBox);
            document.body.removeChild(button);
            resolve(communication);
        };

        let state = "ready";
        button.onclick = async () => {
            switch (state) {
                case "ready":
                    state = "working";
                    if (inputBox.value) {
                        button.textContent = "Working";
                        let answer = await communication.connect(JSON.parse(inputBox.value));
                        inputBox.value = JSON.stringify(answer);
                        button.textContent = "Waiting";
                    } else {
                        let offer = await communication.serve();
                        inputBox.value = JSON.stringify(offer);
                        state = "serving";
                    }
                    
                    break;

                case "serving":
                    state = "working";
                    let answer = JSON.parse(inputBox.value);
                    communication.connection.setRemoteDescription(answer);

                    break;
                    

                default:
                    console.log("Weird state: " + state);
            }
        };
    } catch (e) {
        reject(e);
    }
}