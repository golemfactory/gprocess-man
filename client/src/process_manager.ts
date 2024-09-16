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
        const proc = new Process(this.#client, resp.start);
        this.#processes.push(proc);
        return proc;
    }

    ps(): Process[] {
        // TODO: implement this in protocol
        return this.#processes;
    }

    killAll(): Promise<number[]> {
        return Promise.all(this.ps().map(p => p.kill()));
    }


    #client: Client;
    #processes: Process[] = [];
}
