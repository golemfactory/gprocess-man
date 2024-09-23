import * as net from "node:net";
import { Socket } from ".";

export class TcpSocket implements Socket {
    constructor(port: number, host?: string) {
        this.#socket = net.connect(port, host);
    }

    onClose(callback: () => void) {
        return this.#socket.on("close", callback);
    }

    onData(callback: (data: Uint8Array) => void) {
        return this.#socket.on("data", callback);
    }

    onError(callback: (error: Error) => void) {
        return this.#socket.on("error", callback);
    }

    write(data: Uint8Array) {
        return this.#socket.write(data);
    }

    close() {
        return this.#socket.end();
    }

    #socket: net.Socket;
}
