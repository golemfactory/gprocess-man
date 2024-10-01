import { Socket } from ".";

export class WsSocket implements Socket {
    static create(uri: string, key: string): Promise<WsSocket> {
        const url = new URL(uri);
        url.searchParams.append("authToken", key);
        return new Promise(resolve => {
            const socket = new WebSocket(url.href);
            socket.binaryType = "arraybuffer";
            socket.onopen = () => resolve(new WsSocket(socket));
        });
    }

    constructor(socket: WebSocket) {
        this.#socket = socket;
    }

    onClose(callback: () => void) {
        this.#socket.onclose = callback;
    }

    onData(callback: (data: Uint8Array) => void) {
        this.#socket.onmessage = e => {
            let data = null;
            if (typeof e.data === "string") {
                data = new TextEncoder().encode(e.data);
            } else {
                data = new Uint8Array(e.data);
            }
            callback(data);
        };
    }

    onError(callback: (error: Error) => void) {
        this.#socket.onerror = e => callback(new Error(`${e}`));
    }

    write(data: Uint8Array) {
        return this.#socket.send(data);
    }

    close() {
        return this.#socket.close();
    }

    #socket: WebSocket;
}
