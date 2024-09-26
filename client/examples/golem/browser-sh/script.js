import { TaskExecutor } from "https://unpkg.com/@golem-sdk/task-executor";
import { ProcessManager, WsSocket } from "./gprocess-man.js";


const input = document.getElementById("input");
const output = document.getElementById("output");
const button = document.getElementById("button");

let starting = false;
let exe = null;
let procman = null;
let shell = null;
let shellStdoutStop = null;
let shellStderrStop = null;


function log(message) {
    console.info(message);
    input.contentEditable = false;
    input.innerText = message;
}

function info(prefix, data) {
    if (data == null) {
        return;
    }
    if (typeof data !== "string") {
        data = new TextDecoder().decode(data);
    }
    console.info(data.replace(/^./gm, `${prefix}: $&`));
}

async function start() {
    if (exe != null || procman != null || shell != null ||
        shellStdoutStop != null || shellStderrStop != null) {
        console.error("Cannot start, already running");
        throw new Error("Cannot start, already running");
    }

    log("Starting image...");
    // TODO: add gprocess-man image tag?
    const imageHash = "68d1c6a972a26d49c2a95a3e6d04adf5bd292966cd3f3c62ed94047f";

    exe = await TaskExecutor.create({
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

    exe.run(async exe => {
        log("Starting gprocess-man...");

        // FIXME: sleep 1 in order to not miss logs
        const cmd = await exe.runAndStream("sleep 1; gprocess-man");
        cmd.stdout.subscribe(data => info("gprocess-man", data));
        cmd.stderr.subscribe(data => info("gprocess-man", data));

        // Wait for gprocess-man to start and listen to connections
        await new Promise(r => setTimeout(r, 1 * 1000));

        log("Connecting to gprocess-man...");
        const wsUri = exe.getWebsocketUri(1234);
        procman = new ProcessManager(await WsSocket.create(wsUri, "try_golem"));

        log("Starting shell...");
        shell = await procman.run({
            program: "sh",
            args: [],
            env: [],
        });

        log("Handling shell stderr...");
        const len = 1024 * 1024;
        const td = new TextDecoder();
        new Promise(async resolve => {
            shellStderrStop = resolve;
            while (shell != null) {
                const buf = await shell.read(len, 2);
                output.innerText += td.decode(buf);
                await new Promise(r => setTimeout(r, 500));
            }
        });

        log("Handling shell stdout...");
        new Promise(async resolve => {
            shellStdoutStop = resolve;
            while (shell != null) {
                const buf = await shell.read(len, 1);
                output.innerText += td.decode(buf);
                await new Promise(r => setTimeout(r, 500));
            }
        });

        log("Handling shell stdin...");
        input.innerText = "";
        input.contentEditable = true;
        input.focus();
    });
}

async function stop() {
    if (shellStdoutStop != null) {
        log("Stopping shell stdout...");
        shellStdoutStop();
        shellStdoutStop = null;
    }

    if (shellStderrStop != null) {
        log("Stopping shell stderr...");
        shellStderrStop();
        shellStderrStop = null;
    }

    if (shell != null) {
        log("Stopping shell stdin...");
        shell = null;
    }

    if (procman != null) {
        log("Killing shell...");
        await procman.close();
        procman = null;
    }

    if (exe != null) {
        log("Killing image...");
        await exe.shutdown();
        exe = null;
    }

    log("Not running...");
}


input.onkeydown = e => {
    if (shell != null && e.keyCode === 13 && !e.shiftKey) {
        e.preventDefault();
        const cmd = input.innerText.replace(/\n*$/, "");
        input.innerText = "";
        shell.write(`printf '\\n$ %s\\n' '${cmd}'\n${cmd}\n`);
    }
};

button.onclick = async () => {
    if (starting || exe != null || procman != null || shell != null ||
        shellStdoutStop != null || shellStderrStop != null) {
        await stop();
        button.innerText = "Start";
        starting = false;
    } else {
        starting = true;
        button.innerText = "Stop";
        start();
    }
};
