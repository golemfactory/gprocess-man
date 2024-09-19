import { ProcessManager } from "#lib";

function str(value: Uint8Array): string {
    return new TextDecoder().decode(value);
}

async function example_cat() {
    const procman = new ProcessManager();
    const proc = await procman.run({
        program: "cat",
        args: [],
        env: [],
    });
    proc.onExit(status => console.log("EXIT:", status));
    await proc.write("ping");
    await proc.write("pong");
    const output = str(await proc.read());
    console.log("OUTPUT:", output);
    await procman.close();
}

example_cat().catch(e => console.error("Error:", e.message));
