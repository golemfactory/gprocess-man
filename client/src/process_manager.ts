import { StartRequest, Stream } from "./gprocess";
import { Client } from "./client";
import { Process } from "./process";

export class ProcessManager {
    constructor() {
        this.#client = new Client();
    }

    async close() {
        await this.killAll();
        this.#client.close();
    }

    closeForce() {
        return this.killAll().finally(() => this.#client.close());
    }

    async run(start: StartRequest): Promise<Process> {
        start = {
            ...start,
            stdin: Stream.PIPE,
            stdout: Stream.PIPE,
            stderr: Stream.PIPE,
        };
        const resp = await this.#client.send({ $case: "start", start });
        if (resp?.$case !== "start") {
            console.error("Error running new process:", start);
            throw new Error(`Invalid response type to a start request: ${resp?.$case}`);
        }
        console.debug("Spawned new process:", resp.start);
        return new Process(this.#client, resp.start);
    }

    async ps(): Promise<Process[]> {
        const resp = await this.#client.send({ $case: "ps", ps: {} });
        if (resp?.$case !== "ps") {
            throw new Error(`Invalid response type to a ps request: ${resp?.$case}`);
        }
        // TODO: add stdio info to ps
        return resp.ps.pid.map(pid => new Process(this.#client, { pid }));
    }

    async killAll(): Promise<number[]> {
        const ps = await this.ps();
        return Promise.all(ps.map(p => p.kill()));
    }

    #client: Client;
}
