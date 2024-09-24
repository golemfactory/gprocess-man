import { WebSocket } from "ws";
import { Socket } from ".";

export class WsSocket implements Socket {
    static create(uri: string, key: string): Promise<WsSocket> {
        return new Promise(resolve => {
            const socket = new WebSocket(uri, {
                headers: {
                    authorization: `Bearer ${key}`,
                }
            });
            socket.on("open", () => resolve(new WsSocket(socket)));
        });
    }

    constructor(socket: WebSocket) {
        this.#socket = socket;
    }

    onClose(callback: () => void) {
        return this.#socket.on("close", callback);
    }

    onData(callback: (data: Uint8Array) => void) {
        return this.#socket.on("message", callback);
    }

    onError(callback: (error: Error) => void) {
        return this.#socket.on("error", callback);
    }

    write(data: Uint8Array) {
        return this.#socket.send(data);
    }

    close() {
        return this.#socket.close();
    }

    #socket: WebSocket;
}
