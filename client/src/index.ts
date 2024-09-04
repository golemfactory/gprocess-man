import * as gprocess from "./gprocess.ts";
import * as pb from "protobufjs/minimal";
import { Socket } from "bun";
import {
  Error,
  ReadResponse,
  SignalResponse,
  StartResponse,
  WaitResponse,
  WriteResponse,
} from "./gprocess.ts";

const _te = new TextEncoder();

function str(value: string | Uint8Array): Uint8Array {
  if (typeof value === "string") {
    return _te.encode(value);
  }
  return value;
}

type Command =
  | { start: gprocess.StartRequest }
  | { signal: gprocess.SignalRequest }
  | { wait: gprocess.WaitRequest }
  | { read: gprocess.ReadRequest }
  | { write: gprocess.WriteRequest };

type CommandResponse =
  | { start: gprocess.StartResponse }
  | { signal: gprocess.SignalResponse }
  | { wait: gprocess.WaitResponse }
  | { read: gprocess.ReadResponse }
  | { write: gprocess.WriteResponse }
  | { error: Error };

class PacketDecoder {
  bufSize : number;
  buffer : Uint8Array;
  pos : number;
  cb : undefined | ((r: gprocess.Response) => void);

  constructor(bufSize) {
    this.buffer = new Uint8Array(bufSize);
    this.pos = 0;
    this.bufSize = bufSize;
  }

  ensure(size) {
    if (this.buffer.length >= size + this.pos) {
      return;
    }
    let nextSize = Math.max(this.buffer.length*2, this.bufSize);
    while (nextSize < size + this.pos) {
      nextSize *= 2;
    }
    const nextBuffer = new Uint8Array(nextSize);
    nextBuffer.set(this.buffer.slice(0, this.pos));
    this.buffer = nextBuffer;
  }

  consume(data : Uint8Array) {
    this.ensure(data.length);
    this.buffer.set(data, this.pos)
    this.pos += data.length;

    for (; ;) {
      if (this.pos >= 4) {
        const size = this.buffer[3] + (this.buffer[2] << 8) +
            (this.buffer[1] << 16);
        if (this.pos < size+4) {
          break;
        }
        this.decode(this.buffer.slice(4, size + 4));
        if (this.pos == size + 4) {
          this.pos = 0;
          return;
        }
        this.buffer = this.buffer.slice(size+4);
        this.pos -= size+4;
      }
      else {
        break;
      }
    }

  }

  decode(packet : Uint8Array) {
    const resp = gprocess.Response.decode(packet);
    if (this.cb) {
      const cb = this.cb;
      cb(resp);
    }
  }

  onPacket(cb: (r: gprocess.Response) => void) {
    this.cb = cb;
  }

}

class Client {
  socket: Socket | null = null;
  request_id = 1;
  decoder = new PacketDecoder(1024);

  constructor() {
    this.decoder.onPacket((r) => {
      console.log('resp=', r);
    })
  }

  async start() {
    this.socket = await Bun.connect({
      hostname: "127.0.0.1",
      port: 1234,
      socket: {
        data: (socket, data) => this.data(socket, data),
        binaryType: "uint8array",
      },
    });
  }

  async data(socket, data: Uint8Array) {
    this.decoder.consume(data);
  }

  async send(cmd: Command) {
    const requestId = this.request_id++;
    const $case = Object.keys(cmd)[0];
    const request = {
      requestId,
      command: { $case, ...cmd },
    };
    let data = gprocess.Request.encode(request as any).finish();

    let sd = new ArrayBuffer(4);
    new DataView(sd).setUint32(0, data.length, false);
    const s = gprocess.Request.decode(data);
    await this.socket?.write(sd);
    await this.socket?.write(data);
  }
}

const client = new Client();
await client.start();

await client.send({
  start: {
    program: "/usr/bin/find",
    args: [str("/home/reqc/workspace/ya/gprocess-man")],
    env: [],
    stdin: gprocess.Stream.PIPE,
    stdout: gprocess.Stream.INHERIT,
    stderr: gprocess.Stream.INHERIT,
  },
});

//await client.socket?.shutdown();
