#!/usr/bin/env node
/**
 * MCP Bridge — Node.js process managing MCP servers via @modelcontextprotocol/sdk.
 *
 * Protocol (NDJSON via stdin/stdout):
 *   READ:  { "id": N, "method": "...", "params": {...} }
 *   WRITE: { "id": N, "result": {...} } | { "id": N, "error": { "message": "..." } }
 *
 * Commands:
 *   connect     — spawn/connect an MCP server
 *   disconnect  — shut down all servers
 *   listTools   — list tools from one server
 *   callTool    — call a tool on one server
 *   getConfig   — return current config
 *   updateConfig — update config, reconnect all
 */

import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import { createInterface } from "readline";

const registry = new Map();
const serverConfigs = {};

function emitstderr(msg) {
  process.stderr.write(`[mcp-proxy] ${msg}\n`);
}

function sendReply(id, payload) {
  process.stdout.write(JSON.stringify({ id, result: payload }) + "\n");
}

function sendError(id, msg) {
  process.stdout.write(JSON.stringify({ id, error: { message: msg } }) + "\n");
}

async function establishConnection(serverName, cfg) {
  emitstderr(`establishing: ${serverName} → ${cfg.command} ${(cfg.args || []).join(" ")}`);

  try {
    const transport = new StdioClientTransport({
      command: cfg.command,
      args: cfg.args || [],
      env: { ...process.env, ...(cfg.env || {}) },
      cwd: cfg.cwd || undefined,
    });

    const client = new Client(
      { name: "qwen-studio-mcp", version: "1.0.0" },
      { capabilities: {} }
    );

    await client.connect(transport, { timeout: 60000 });
    registry.set(serverName, client);
    emitstderr(`established: ${serverName}`);
    return { connected: true };
  } catch (err) {
    emitstderr(`establish FAILED ${serverName}: ${err.message}`);
    throw err;
  }
}

async function fetchToolList(serverName) {
  const client = registry.get(serverName);
  if (!client) {
    return { tools: [] };
  }
  const response = await client.listTools();
  return {
    tools: response.tools.map((t) => ({
      name: t.name,
      description: t.description,
      inputSchema: t.inputSchema,
    })),
  };
}

async function executeToolCall(serverName, toolName, toolArgs) {
  emitstderr(`executeToolCall: ${serverName}#${toolName}`);
  const client = registry.get(serverName);
  if (!client) {
    emitstderr(`executeToolCall: ${serverName} offline`);
    return {
      content: [
        {
          type: "text",
          text: `Server "${serverName}" is offline. Activate it in MCP settings.`,
        },
      ],
    };
  }
  const response = await client.callTool({
    name: toolName,
    arguments: toolArgs || {},
  });
  emitstderr(`executeToolCall: ${serverName}#${toolName} done`);
  return response;
}

async function terminateAll() {
  emitstderr("terminating all connections");
  for (const [name, client] of registry) {
    try {
      await client.close();
      emitstderr(`terminated: ${name}`);
    } catch (err) {
      emitstderr(`termination error ${name}: ${err.message}`);
    }
  }
  registry.clear();
}

function configKey(cfg) {
  return JSON.stringify({
    command: cfg.command,
    args: cfg.args || [],
    env: cfg.env || {},
    cwd: cfg.cwd || undefined,
  });
}

async function reconcileConfig(newConfigs) {
  const next = newConfigs || {};
  const nextNames = new Set(Object.keys(next));
  const currentNames = Object.keys(serverConfigs);
  emitstderr(`reconcileConfig: current=[${currentNames.join(",")}] next=[${[...nextNames].join(",")}]`);

  // 1. Disconnect servers that are no longer present in the new config.
  for (const name of currentNames) {
    if (!nextNames.has(name)) {
      const client = registry.get(name);
      if (client) {
        try {
          await client.close();
          emitstderr(`disconnected removed: ${name}`);
        } catch (err) {
          emitstderr(`disconnect error ${name}: ${err.message}`);
        }
        registry.delete(name);
      }
      delete serverConfigs[name];
    }
  }

  // 2. Connect new servers, reconnect changed ones, leave unchanged alone.
  // Compare only functional fields (command/args/env/cwd) so metadata-only
  // pushes (e.g. added "source"/"from") don't tear down a live server.
  for (const [name, cfg] of Object.entries(next)) {
    const existing = serverConfigs[name];
    const changed =
      !existing || configKey(existing) !== configKey(cfg);

    if (!changed && registry.has(name)) {
      // Already connected with identical functional config — keep it alive (no-op).
      emitstderr(`reconcileConfig: ${name} unchanged (kept alive)`);
      continue;
    } else if (changed) {
      emitstderr(`reconcileConfig: ${name} changed (will reconnect)`);
    }

    if (registry.has(name)) {
      try {
        await registry.get(name).close();
      } catch (err) {
        emitstderr(`disconnect error ${name}: ${err.message}`);
      }
      registry.delete(name);
    }

    try {
      await establishConnection(name, cfg);
      emitstderr(`(re)established: ${name}`);
    } catch (err) {
      emitstderr(`establish FAILED ${name}: ${err.message}`);
    }
  }

  Object.assign(serverConfigs, next);
  emitstderr(`reconcileConfig done: [${Object.keys(serverConfigs).join(",")}]`);
  return structuredClone(serverConfigs);
}

const inputReader = createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

emitstderr("bridge process initialized");

inputReader.on("line", async (rawLine) => {
  let envelope;
  try {
    envelope = JSON.parse(rawLine);
  } catch {
    return;
  }

  const { id, method, params = {} } = envelope;

  try {
    let output;
    switch (method) {
      case "connect":
        output = await establishConnection(params.serverName, params.config);
        break;
      case "disconnect":
        await terminateAll();
        output = { disconnected: true };
        break;
      case "listTools":
        output = await fetchToolList(params.serverName);
        break;
      case "callTool":
        output = await executeToolCall(
          params.serverName,
          params.toolName,
          params.toolArguments
        );
        break;
      case "getConfig":
        output = structuredClone(serverConfigs);
        break;
      case "updateConfig":
        output = await reconcileConfig(params.config);
        break;
      default:
        throw new Error(`Unsupported method: ${method}`);
    }
    sendReply(id, output);
  } catch (err) {
    sendError(id, err.message);
  }
});

inputReader.on("close", () => {
  emitstderr("input stream closed, initiating shutdown");
  terminateAll().then(() => process.exit(0));
});
