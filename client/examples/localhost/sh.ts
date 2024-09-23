import { ProcessManager, TcpSocket } from "#lib";

function str(value: Uint8Array): string {
    return new TextDecoder().decode(value);
}

async function example_sh() {
    const procman = new ProcessManager(new TcpSocket(1234));
    const proc = await procman.run({
        program: "sh",
        args: [],
        env: [],
    });
    proc.onExit(status => console.log("EXIT:", status));
    await proc.write("echo test_sh\n");
    const output = str(await proc.read());
    console.log("OUTPUT:", output);
    await procman.close();
}

example_sh().catch(e => console.error("Error:", e.message));
