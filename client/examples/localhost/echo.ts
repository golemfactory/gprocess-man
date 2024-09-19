import { ProcessManager } from "#lib";

function str(value: Uint8Array): string {
    return new TextDecoder().decode(value);
}

function args(value: string[]): Uint8Array[] {
    const te = new TextEncoder();
    return value.map(x => te.encode(x));
}

async function example_echo() {
    const procman = new ProcessManager();
    const proc = await procman.run({
        program: "echo",
        args: args(["test"]),
        env: [],
    });
    proc.onExit(status => console.log("EXIT:", status));
    const output = str(await proc.read());
    console.log("OUTPUT:", output);
    await procman.close();
}

example_echo().catch(e => console.error("Error:", e.message));
