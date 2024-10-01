import { Client } from "./client";
import { StartRequest, Stream } from "./gprocess";
import { Process } from "./process";
import { Socket } from "./socket";

export class ProcessManager {
    constructor(socket: Socket, timeout_ms: number = 30 * 1000) {
        this.#client = new Client(socket, timeout_ms);
    }

    close() {
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
        this.#processes.set(proc.info.pid, proc);
        return proc;
    }

    async ps(): Promise<number[]> {
        const resp = await this.#client.send({ $case: "ps", ps: {} });
        if (resp?.$case !== "ps") {
            throw new Error(`Invalid response type to a ps request: ${resp?.$case}`);
        }
        return resp.ps.pid;
    }

    async killAll(): Promise<(number | void)[]> {
        const ps = Array.from(this.#processes.values());
        return Promise.all(ps.map(p => p.kill()));
    }

    #client: Client;
    #processes: Map<number, Process> = new Map();
}
