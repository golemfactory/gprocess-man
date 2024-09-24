import { TaskExecutor } from "@golem-sdk/task-executor";
import { pinoPrettyLogger } from "@golem-sdk/pino-logger";
import { ProcessManager, WsSocket } from "#lib";

function str(value: Uint8Array): string {
    return new TextDecoder().decode(value);
}

function info(prefix: string, data: string | ArrayBuffer | null | undefined) {
    if (data == null) {
        return;
    }
    if (typeof data !== "string") {
        data = new TextDecoder().decode(data);
    }
    console.info(data.replace(/^./gm, `${prefix}: $&`));
}

async function example_sh() {
    // TODO: add gprocess-man image tag?
    const imageHash = "68d1c6a972a26d49c2a95a3e6d04adf5bd292966cd3f3c62ed94047f";

    const exe = await TaskExecutor.create({
        logger: pinoPrettyLogger(),
        api: { key: "try_golem" },
        demand: { workload: { imageHash } },
        market: {
            rentHours: 0.5,
            pricing: {
                model: "linear",
                maxStartPrice: 0.5,
                maxCpuPerHourPrice: 1.0,
                maxEnvPerHourPrice: 0.5,
            },
        },
        vpn: true,
    });

    await exe.run(async exe => {
        // FIXME: sleep 1 in order to not miss logs
        const cmd = await exe.runAndStream("sleep 1; gprocess-man");
        cmd.stdout.subscribe(data => info("gprocess-man", data));
        cmd.stderr.subscribe(data => info("gprocess-man", data));

        // Wait for gprocess-man to start and listen to connections
        await new Promise(r => setTimeout(r, 1 * 1000));

        const wsUri = exe.getWebsocketUri(1234);
        const procman = new ProcessManager(await WsSocket.create(wsUri, "try_golem"));
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
    });

    await exe.shutdown();
}

example_sh().catch(e => console.error("Error:", e.message));
