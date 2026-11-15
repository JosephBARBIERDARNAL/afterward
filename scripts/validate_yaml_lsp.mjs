// Checks the syntax of every game data YAML file with yaml-language-server
// (https://github.com/redhat-developer/yaml-language-server), the server the
// editors run. It has no command line of its own, so this speaks the Language
// Server Protocol to it over stdio: open every file, collect the diagnostics it
// publishes, and fail on any error. Needs `yaml-language-server` on the PATH.

import { spawn } from "node:child_process";
import { readFileSync, globSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const files = globSync("src/assets/**/*.{yaml,yml}").sort();
if (files.length === 0) {
  console.error("No game data YAML files found");
  process.exit(1);
}

const settings = {
  yaml: { validate: true, schemas: {}, schemaStore: { enable: false }, customTags: [] },
};

const server = spawn("yaml-language-server", ["--stdio"], { stdio: ["pipe", "pipe", "inherit"] });
server.on("error", (error) => {
  console.error(`Could not start yaml-language-server: ${error.message}`);
  process.exit(1);
});

let nextId = 1;
const pending = new Map();
const send = (message) => {
  const body = JSON.stringify({ jsonrpc: "2.0", ...message });
  server.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
};
const request = (method, params) =>
  new Promise((done) => {
    const id = nextId++;
    pending.set(id, done);
    send({ id, method, params });
  });

const uris = new Map(files.map((path) => [pathToFileURL(resolve(path)).href, path]));
const diagnostics = new Map();
let allReported;
const reported = new Promise((done) => (allReported = done));

const onMessage = (message) => {
  if (message.id !== undefined && message.method === undefined) {
    pending.get(message.id)?.(message.result);
    pending.delete(message.id);
  } else if (message.method === "workspace/configuration") {
    send({ id: message.id, result: message.params.items.map(() => settings.yaml) });
  } else if (message.id !== undefined) {
    send({ id: message.id, result: null });
  } else if (message.method === "textDocument/publishDiagnostics" && uris.has(message.params.uri)) {
    diagnostics.set(message.params.uri, message.params.diagnostics);
    if (diagnostics.size === uris.size) allReported();
  }
};

let buffer = Buffer.alloc(0);
server.stdout.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (;;) {
    const headerEnd = buffer.indexOf("\r\n\r\n");
    if (headerEnd < 0) return;
    const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, headerEnd).toString())[1]);
    if (buffer.length < headerEnd + 4 + length) return;
    onMessage(JSON.parse(buffer.subarray(headerEnd + 4, headerEnd + 4 + length).toString()));
    buffer = buffer.subarray(headerEnd + 4 + length);
  }
});

await request("initialize", {
  processId: process.pid,
  rootUri: pathToFileURL(process.cwd()).href,
  capabilities: { workspace: { configuration: true } },
});
send({ method: "initialized", params: {} });
send({ method: "workspace/didChangeConfiguration", params: { settings } });
for (const [uri, path] of uris) {
  send({
    method: "textDocument/didOpen",
    params: { textDocument: { uri, languageId: "yaml", version: 1, text: readFileSync(path, "utf8") } },
  });
}

const timeout = setTimeout(() => {
  const silent = [...uris].filter(([uri]) => !diagnostics.has(uri)).map(([, path]) => path);
  console.error(`yaml-language-server reported nothing for: ${silent.join(", ")}`);
  process.exit(1);
}, 60_000);
await reported;
clearTimeout(timeout);

let errors = 0;
for (const [uri, path] of uris) {
  for (const { severity, range, message } of diagnostics.get(uri)) {
    if (severity !== 1) continue;
    errors++;
    console.error(`${path}:${range.start.line + 1}:${range.start.character + 1}: ${message}`);
  }
}

await request("shutdown", null);
send({ method: "exit" });
if (errors > 0) process.exit(1);
console.log(`yaml-language-server validated ${files.length} game data YAML files`);
