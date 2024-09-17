import { ProcessManager } from "./index";

function u8a(value: string): Uint8Array {
    return new TextEncoder().encode(value);
}

function str(value: Uint8Array): string {
    return new TextDecoder().decode(value);
}

function args(value: string[]): Uint8Array[] {
    const te = new TextEncoder();
    return value.map(x => te.encode(x));
}

async function example1() {
    const proc = await procman.run({
        program: "echo",
        args: args(["test"]),
        env: [],
    });
    const output = str(await proc.read());
    console.log("EXAMPLE1:", output);
}

async function example2() {
    const proc = await procman.run({
        program: "cat",
        args: [],
        env: [],
    });
    await proc.write(u8a("ping"));
    await proc.write(u8a("pong"));
    const output = str(await proc.read());
    console.log("EXAMPLE2:", output);
}

async function example3() {
    const proc = await procman.run({
        program: "sh",
        args: [],
        env: [],
    });
    await proc.write(u8a("echo test_sh\n"));
    const output = str(await proc.read());
    console.log("EXAMPLE3:", output);
}

const procman = new ProcessManager();
Promise.all([example1(), example2(), example3()])
    .catch(e => console.error("Error:", e.message))
    .finally(() => procman.close().catch(e => console.error(e.message)));
