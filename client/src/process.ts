import { StartResponse } from "./gprocess";
import { Client } from "./client";

export class Process {
    constructor(client: Client, info: StartResponse) {
        this.#client = client;
        this.info = info;
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
        // TODO: remove len from response
        return resp.read.data.subarray(0, resp.read.len);
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

    async wait(timeout_ms: number = -1): Promise<number> {
        const wait = {
            pid: this.info.pid,
        };
        const resp = await this.#client.send({ $case: "wait", wait }, timeout_ms);
        if (resp?.$case !== "wait") {
            throw new Error(`Invalid response type to a wait request: ${resp?.$case}`);
        }
        return resp.wait.status;
    }

    kill(timeout_ms: number = 5 * 1000): Promise<number> {
        this.signal(15).catch(() => {});
        const timeoutId = setTimeout(
            () => this.signal(9).catch(() => {}),
            timeout_ms,
        );
        return this.wait(timeout_ms * 2)
            .finally(() => clearTimeout(timeoutId));
    }


    info: StartResponse;
    #client: Client;
}
