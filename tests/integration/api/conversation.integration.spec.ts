import axios, { AxiosInstance, type AxiosRequestConfig } from "axios";
import { ChildProcess, spawn } from "node:child_process";
import process from "node:process";
import { setTimeout as delay } from "node:timers/promises";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

function resolveBaseUrlFromConfig(): string {
  // Use the same host/port settings as the Rust service, falling back to loopback when necessary.
  const rawHost = process.env.SERVER_HOST?.trim() ?? "127.0.0.1";
  const normalizedHost =
    rawHost === "0.0.0.0" || rawHost === "::" || rawHost === "[::]" ? "127.0.0.1" : rawHost;

  const rawPort = process.env.SERVER_PORT?.trim();
  const parsedPort = rawPort ? Number.parseInt(rawPort, 10) : undefined;
  const port =
    parsedPort && Number.isFinite(parsedPort) && parsedPort > 0 && parsedPort <= 65535
      ? String(parsedPort)
      : "8080";

  const needsBrackets = normalizedHost.includes(":") && !normalizedHost.startsWith("[");
  const hostForUrl = needsBrackets ? `[${normalizedHost}]` : normalizedHost;

  return `http://${hostForUrl}:${port}`;
}

const explicitApiBaseUrl = process.env.API_BASE_URL?.trim();
const BASE_URL = explicitApiBaseUrl?.length ? explicitApiBaseUrl : resolveBaseUrlFromConfig();
const STARTUP_TIMEOUT_MS = Number(process.env.API_STARTUP_TIMEOUT ?? 30_000);
const USE_EXTERNAL_API = Boolean(explicitApiBaseUrl?.length);

let serverProcess: ChildProcess | undefined;
let httpClient: AxiosInstance;

interface ConversationResponse {
  conversation_id: string;
  title: string;
  description: string | null;
  created_at: string;
  created_by: string;
  is_public: boolean;
  fork_from_conversation_id: string | null;
  fork_from_message_id: string | null;
}

interface MessageResponse {
  conversation_id: string;
  message_id: string;
  parent_message_id: string | null;
  role: string;
  content: {
    type: string;
    [key: string]: unknown;
  };
  content_metadata: Record<string, string>;
  lineage: string[];
  depth: number;
  created_at: string;
  created_by: string;
}

interface TreeResponse {
  conversation_id: string;
  messages: MessageResponse[];
  total_messages: number;
}

async function waitForHealth(baseUrl: string, timeoutMs: number): Promise<void> {
  const startedAt = Date.now();

  while (Date.now() - startedAt < timeoutMs) {
    try {
      const requestConfig: AxiosRequestConfig = {
        timeout: 2_000,
        validateStatus: () => true
      };

      if (!USE_EXTERNAL_API) {
        requestConfig.proxy = false;
      }

      const response = await axios.get(`${baseUrl}/health`, requestConfig);

      if (response.status === 200 && response.data?.status === "ok") {
        return;
      }
    } catch {
      // Back off and retry
    }

    await delay(500);
  }

  throw new Error(`Timed out after ${timeoutMs}ms waiting for API health check at ${baseUrl}/health`);
}

async function startServer(): Promise<void> {
  if (USE_EXTERNAL_API) {
    await waitForHealth(BASE_URL, STARTUP_TIMEOUT_MS);
    return;
  }

  const collectedOutput: string[] = [];

  serverProcess = spawn("cargo", ["run"], {
    cwd: process.cwd(),
    env: {
      ...process.env,
      RUST_LOG: process.env.RUST_LOG ?? "info"
    },
    stdio: ["ignore", "pipe", "pipe"]
  });

  const outputHandler = (data: Buffer) => {
    const text = data.toString();
    text
      .trim()
      .split(/\r?\n/)
      .filter(Boolean)
      .forEach((line) => collectedOutput.push(`[stdout] ${line}`));

    const prefix = "[api] ";
    if (process.env.VITEST_POOL_ID) {
      // When running under Vitest worker pools, prefix helps separate output.
      text.trim()
        .split(/\r?\n/)
        .forEach((line) => process.stdout.write(`${prefix}${line}\n`));
    } else if (process.env.API_TEST_DEBUG === "1") {
      process.stdout.write(`${prefix}${text}`);
    }
  };

  serverProcess.stdout?.on("data", outputHandler);
  serverProcess.stderr?.on("data", (data: Buffer) => {
    const text = data.toString();
    text
      .trim()
      .split(/\r?\n/)
      .filter(Boolean)
      .forEach((line) => collectedOutput.push(`[stderr] ${line}`));

    text
      .trim()
      .split(/\r?\n/)
      .forEach((line) => process.stderr.write(`[api:err] ${line}\n`));
  });

  if (!serverProcess) {
    throw new Error("Failed to spawn API process");
  }

  let exitHandler: ((code: number | null, signal: NodeJS.Signals | null) => void) | undefined;

  const child: ChildProcess = serverProcess;

  const exitDuringStartup = new Promise<never>((_, reject) => {
    exitHandler = (code: number | null, signal: NodeJS.Signals | null) => {
      reject(
        new Error(
          `API process exited before ready (code=${code ?? "null"}, signal=${signal ?? "null"})\n--- API output ---\n${collectedOutput.join(
            "\n"
          )}`
        )
      );
    };

    child.once("exit", exitHandler);
  });

  try {
    await Promise.race([waitForHealth(BASE_URL, STARTUP_TIMEOUT_MS), exitDuringStartup]);
  } catch (error) {
    if (exitHandler) {
      child.off("exit", exitHandler);
    }

    if (child.exitCode === null) {
      child.kill("SIGKILL");
      await new Promise<void>((resolve) => child.once("exit", () => resolve()));
    }

    serverProcess = undefined;
    throw error;
  }

  if (exitHandler) {
    child.off("exit", exitHandler);
  }
}

async function stopServer(): Promise<void> {
  if (!serverProcess) {
    return;
  }

  const child = serverProcess;
  serverProcess = undefined;

  const exitPromise = new Promise<void>((resolve) => {
    child.once("exit", () => resolve());
  });

  if (process.platform === "win32") {
    spawn("taskkill", ["/pid", String(child.pid), "/t", "/f"]);
  } else {
    child.kill("SIGINT");
    setTimeout(() => {
      if (!child.killed) {
        child.kill("SIGKILL");
      }
    }, 5_000);
  }

  await exitPromise;
}

beforeAll(async () => {
  await startServer();

  httpClient = axios.create({
    baseURL: BASE_URL,
    timeout: 10_000,
    validateStatus: () => true,
    ...(USE_EXTERNAL_API ? {} : { proxy: false as const })
  });
});

afterAll(async () => {
  await stopServer();
});

describe("Conversations API", () => {
  it("creates a conversation, appends a message, and retrieves it", async () => {
    const createPayload = {
      title: `Integration Conversation ${Date.now()}`,
      created_by: "integration-test-user"
    };

    const createResponse = await httpClient.post<ConversationResponse>(
      "/api/v1/conversations",
      createPayload
    );

    expect(createResponse.status).toBe(200);
    expect(createResponse.data.conversation_id).toBeDefined();
    expect(createResponse.data.title).toBe(createPayload.title);
    expect(createResponse.data.created_by).toBe(createPayload.created_by);

    const conversationId = createResponse.data.conversation_id;

    const treeResponse = await httpClient.get<TreeResponse>(
      `/api/v1/conversations/${conversationId}/tree`
    );

    console.log('treeResponse', treeResponse.data)
    expect(treeResponse.status).toBe(200);
    expect(treeResponse.data.conversation_id).toBe(conversationId);
    expect(treeResponse.data.total_messages).toBeGreaterThan(0);

    const rootMessage = treeResponse.data.messages.find(
      (msg) => msg.parent_message_id === null
    );

    expect(rootMessage, "root message should exist").toBeDefined();

    if (!rootMessage) {
      throw new Error("Root message missing from conversation tree response");
    }

    const rootMessageId = rootMessage.message_id;

    const createMessageResponse = await httpClient.post<MessageResponse>(
      `/api/v1/conversations/${conversationId}/messages`,
      {
        parent_message_id: rootMessageId,
        role: "human",
        content: {
          type: "text",
          text: "Hello from TypeScript integration test!"
        },
        content_metadata: {},
        created_by: "integration-test-user"
      }
    );

    expect(createMessageResponse.status).toBe(200);
    expect(createMessageResponse.data.role).toBe("human");
    expect(createMessageResponse.data.content.type).toBe("text");

    const newMessageId = createMessageResponse.data.message_id;

    const childrenResponse = await httpClient.get<MessageResponse[]>(
      `/api/v1/conversations/${conversationId}/messages/${rootMessageId}/children`
    );

    expect(childrenResponse.status).toBe(200);
    expect(childrenResponse.data.map((msg) => msg.message_id)).toContain(newMessageId);

    const deleteResponse = await httpClient.delete(`/api/v1/conversations/${conversationId}`);
    expect(deleteResponse.status).toBe(200);
  });
});
