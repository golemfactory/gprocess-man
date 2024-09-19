import { StartResponse } from "./gprocess";
import { Client } from "./client";

export class Process {
    constructor(client: Client, info: StartResponse) {
        this.#client = client;
        this.info = info;
        this.#onExit = this.#wait()
            .then(status => {
                if (this.#onExitFunc != null && status != null) {
                    try {
                        this.#onExitFunc(status);
                    } catch (_e) {}
                }
                return this.#close().catch(() => {});
            })
            .catch(e => {
                console.error("Error waiting for process exit status:", e.message);
            });
    }

    async read(len: number = 1024 * 1024, stream: number = 1): Promise<Uint8Array> {
        const read = {
            pid: this.info.pid,
            stream,
            len,
        };
        const resp = await this.#client.send({ $case: "read", read });
        if (resp?.$case !== "read") {
            throw new Error(`Invalid response type to a read request: ${resp?.$case}`);
        }
        return resp.read.data;
    }

    async write(data: Uint8Array, stream: number = 0): Promise<number> {
        const write = {
            pid: this.info.pid,
            stream,
            data,
        };
        const resp = await this.#client.send({ $case: "write", write });
        if (resp?.$case !== "write") {
            throw new Error(`Invalid response type to a write request: ${resp?.$case}`);
        }
        return resp.write.len;
    }

    async signal(sig: number) {
        const signal = {
            pid: this.info.pid,
            signal: sig,
        };
        const resp = await this.#client.send({ $case: "signal", signal });
        if (resp?.$case !== "signal") {
            throw new Error(`Invalid response type to a signal request: ${resp?.$case}`);
        }
    }

    kill(timeout_ms: number = 5 * 1000): Promise<number | void> {
        this.signal(15).catch(() => {});
        const timeoutId = setTimeout(
            () => this.signal(9).catch(() => {}),
            timeout_ms,
        );
        return this.#onExit
            .finally(() => clearTimeout(timeoutId));
    }

    onExit(callback: ((status: number) => void) | null) {
        this.#onExitFunc = callback;
    }

    async #wait(): Promise<number | void> {
        const wait = {
            pid: this.info.pid,
        };
        const resp = await this.#client.send({ $case: "wait", wait }, -1);
        if (resp?.$case !== "wait") {
            throw new Error(`Invalid response type to a wait request: ${resp?.$case}`);
        }
        if (resp.wait.status == null && resp.wait.alreadyWaits !== true) {
            throw new Error("Invalid response to a wait request: no status");
        }
        return resp.wait.status;
    }

    async #close() {
        const close = {
            pid: this.info.pid,
        };
        const resp = await this.#client.send({ $case: "close", close });
        if (resp?.$case !== "close") {
            throw new Error(`Invalid response type to a close request: ${resp?.$case}`);
        }
    }


    info: StartResponse;
    #client: Client;
    #onExit: Promise<number | void>;
    #onExitFunc: ((status: number) => void) | null = null;
}
