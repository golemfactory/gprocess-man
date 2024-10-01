import { Request, Response } from "./gprocess";
import { Socket } from "./socket";

interface RequestInfo {
    resolve: (response: Response["command"]) => void;
    reject: (reason?: any) => void;
    timeoutId: NodeJS.Timeout | null;
}

export class Client {
    constructor(socket: Socket, timeout_ms: number) {
        this.#socket = socket;
        this.#timeout_ms = timeout_ms;

        this.#socket.onClose(() => this.#onClose());
        this.#socket.onError(e => console.error("Client error:", e.message));
        this.#socket.onData(data => this.#onData(data));
    }

    close() {
        this.#socket.close();
    }

    send(
        command: Request["command"],
        timeout_ms: number = this.#timeout_ms,
    ): Promise<Response["command"]> {
        const requestId = this.#requestId++;
        const data = Request.encode({ requestId, command }).finish();
        const len = new Uint8Array(4);
        new DataView(len.buffer).setUint32(0, data.length);

        this.#socket.write(len);
        this.#socket.write(data);

        return new Promise((resolve, reject) => {
            let timeoutId = null;
            if (timeout_ms >= 0) {
                timeoutId = setTimeout(
                    () => {
                        reject(new Error("Response timeout"));
                        this.#requests.delete(requestId);
                    },
                    timeout_ms,
                );
            }
            this.#requests.set(requestId, { resolve, reject, timeoutId });
        });
    }

    #onData(data: Uint8Array) {
        this.#buf = this.#buf.concat([...data]);

        for (;;) {
            if (this.#buf.length < 4) {
                break;
            }
            const buf = new Uint8Array(this.#buf.slice(0, 4));
            const len = new DataView(buf.buffer).getUint32(0);
            if (this.#buf.length < 4 + len) {
                break;
            }

            this.#buf.splice(0, 4);
            const resp = Response.decode(new Uint8Array(this.#buf.splice(0, len)));
            const req = this.#requests.get(resp.requestId);
            if (req != null) {
                const resolve = req.resolve;
                resolve(resp.command);
                if (req.timeoutId != null) {
                    clearTimeout(req.timeoutId);
                }
                this.#requests.delete(resp.requestId);
            } else {
                console.warn("Response to unknown request:", resp.requestId);
            }
        }
    }

    #onClose() {
        console.debug("Client closed.  Canceling all pending requests...");
        for (const req of this.#requests.values()) {
            req.reject(new Error("Request canceled"));
            if (req.timeoutId != null) {
                clearTimeout(req.timeoutId);
            }
        }
    }


    #socket: Socket;
    #timeout_ms: number;

    #requestId: number = 0;
    #buf: number[] = [];
    #requests: Map<number, RequestInfo> = new Map();
}
