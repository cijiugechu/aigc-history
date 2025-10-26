import axios, { AxiosInstance, type AxiosRequestConfig } from "axios";
import { randomUUID } from "node:crypto";
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

interface BranchResponse {
  conversation_id: string;
  branch_id: string;
  branch_name: string;
  leaf_message_id: string;
  created_at: string;
  last_updated: string;
  created_by: string;
  is_active: boolean;
}

interface ShareResponse {
  conversation_id: string;
  shared_with: string;
  permission: string;
  shared_at: string;
  shared_by: string;
}

type MessageContentPayload = {
  type: string;
  [key: string]: unknown;
};

interface AppendMessagePayload {
  parent_message_id: string;
  role: string;
  content: MessageContentPayload;
  content_metadata?: Record<string, string>;
  created_by: string;
  branch_id?: string;
}

interface CreateConversationOptions {
  title?: string;
  created_by?: string;
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

function uniqueName(prefix: string): string {
  return `${prefix}-${randomUUID()}`;
}

async function createConversation(
  options: CreateConversationOptions = {}
): Promise<ConversationResponse> {
  const payload = {
    title: options.title ?? `Conversation ${uniqueName("ts")}`,
    created_by: options.created_by ?? "ts-integration-user"
  };

  const response = await httpClient.post<ConversationResponse>(
    "/api/v1/conversations",
    payload
  );

  expect(response.status).toBe(200);
  return response.data;
}

async function getConversation(conversationId: string): Promise<ConversationResponse> {
  const response = await httpClient.get<ConversationResponse>(
    `/api/v1/conversations/${conversationId}`
  );
  expect(response.status).toBe(200);
  return response.data;
}

async function getConversationTree(conversationId: string): Promise<TreeResponse> {
  const response = await httpClient.get<TreeResponse>(
    `/api/v1/conversations/${conversationId}/tree`
  );
  expect(response.status).toBe(200);
  return response.data;
}

async function getRootMessage(conversationId: string): Promise<MessageResponse> {
  const tree = await getConversationTree(conversationId);
  const root = tree.messages.find((msg) => msg.parent_message_id === null);
  if (!root) {
    throw new Error("Root message missing from conversation tree response");
  }
  return root;
}

async function appendMessage(
  conversationId: string,
  payload: AppendMessagePayload
): Promise<MessageResponse> {
  const requestBody = {
    parent_message_id: payload.parent_message_id,
    role: payload.role,
    content: payload.content,
    content_metadata: payload.content_metadata ?? {},
    created_by: payload.created_by,
    ...(payload.branch_id ? { branch_id: payload.branch_id } : {})
  };

  const response = await httpClient.post<MessageResponse>(
    `/api/v1/conversations/${conversationId}/messages`,
    requestBody
  );

  expect(response.status).toBe(200);
  return response.data;
}

async function getMessage(
  conversationId: string,
  messageId: string
): Promise<MessageResponse> {
  const response = await httpClient.get<MessageResponse>(
    `/api/v1/conversations/${conversationId}/messages/${messageId}`
  );
  expect(response.status).toBe(200);
  return response.data;
}

async function getMessageChildren(
  conversationId: string,
  messageId: string
): Promise<MessageResponse[]> {
  const response = await httpClient.get<MessageResponse[]>(
    `/api/v1/conversations/${conversationId}/messages/${messageId}/children`
  );
  expect(response.status).toBe(200);
  return response.data;
}

async function getMessageLineage(
  conversationId: string,
  messageId: string
): Promise<MessageResponse[]> {
  const response = await httpClient.get<MessageResponse[]>(
    `/api/v1/conversations/${conversationId}/messages/${messageId}/lineage`
  );
  expect(response.status).toBe(200);
  return response.data;
}

async function createBranch(
  conversationId: string,
  options: { branch_name?: string; leaf_message_id: string; created_by?: string }
): Promise<BranchResponse> {
  const response = await httpClient.post<BranchResponse>(
    `/api/v1/conversations/${conversationId}/branches`,
    {
      branch_name: options.branch_name ?? `branch-${uniqueName("ts")}`,
      leaf_message_id: options.leaf_message_id,
      created_by: options.created_by ?? "ts-branch-user"
    }
  );

  expect(response.status).toBe(200);
  return response.data;
}

async function getBranch(
  conversationId: string,
  branchId: string
): Promise<BranchResponse> {
  const response = await httpClient.get<BranchResponse>(
    `/api/v1/conversations/${conversationId}/branches/${branchId}`
  );
  expect(response.status).toBe(200);
  return response.data;
}

async function getBranchMessages(
  conversationId: string,
  branchId: string
): Promise<MessageResponse[]> {
  const response = await httpClient.get<MessageResponse[]>(
    `/api/v1/conversations/${conversationId}/branches/${branchId}/messages`
  );
  expect(response.status).toBe(200);
  return response.data;
}

async function shareConversationWithUser(
  conversationId: string,
  options: { shared_with: string; permission: "read" | "branch" | "fork"; shared_by?: string }
): Promise<ShareResponse> {
  const response = await httpClient.post<ShareResponse>(
    `/api/v1/conversations/${conversationId}/share`,
    {
      shared_with: options.shared_with,
      permission: options.permission,
      shared_by: options.shared_by ?? "ts-share-user"
    }
  );

  expect(response.status).toBe(200);
  return response.data;
}

async function revokeShareStrict(
  conversationId: string,
  sharedWith: string
): Promise<void> {
  const response = await httpClient.delete(
    `/api/v1/conversations/${conversationId}/shares/${sharedWith}`
  );
  expect(response.status).toBe(200);
}

async function revokeShareIfExists(
  conversationId: string,
  sharedWith: string
): Promise<void> {
  try {
    const response = await httpClient.delete(
      `/api/v1/conversations/${conversationId}/shares/${sharedWith}`
    );

    if (response.status !== 200 && response.status !== 404) {
      throw new Error(
        `Failed to revoke share for ${sharedWith}: HTTP ${response.status}`
      );
    }
  } catch {
    // Ignore cleanup failures.
  }
}

async function deleteConversationIfExists(conversationId: string): Promise<void> {
  try {
    const response = await httpClient.delete(
      `/api/v1/conversations/${conversationId}`
    );

    if (response.status !== 200 && response.status !== 404) {
      throw new Error(
        `Failed to delete conversation ${conversationId}: HTTP ${response.status}`
      );
    }
  } catch {
    // Swallow cleanup errors to avoid masking test failures.
  }
}

describe.sequential("Conversations API", () => {
  it("manages conversation lifecycle metadata", async () => {
    const conversation = await createConversation({
      title: `Lifecycle ${uniqueName("conv")}`,
      created_by: "ts-lifecycle-owner"
    });

    const conversationId = conversation.conversation_id;

    try {
      expect(conversation.title).toContain("Lifecycle");
      expect(conversation.description).toBeNull();
      expect(conversation.fork_from_conversation_id).toBeNull();
      expect(conversation.fork_from_message_id).toBeNull();

      const fetched = await getConversation(conversationId);
      expect(fetched.conversation_id).toBe(conversationId);
      expect(fetched.title).toBe(conversation.title);

      const updatePayload = {
        title: `${conversation.title} Updated`,
        description: "Updated description from TypeScript integration test"
      };

      const updateResponse = await httpClient.put<ConversationResponse>(
        `/api/v1/conversations/${conversationId}`,
        updatePayload
      );
      expect(updateResponse.status).toBe(200);
      expect(updateResponse.data.title).toBe(updatePayload.title);
      expect(updateResponse.data.description).toBe(updatePayload.description);

      const updated = await getConversation(conversationId);
      expect(updated.title).toBe(updatePayload.title);
      expect(updated.description).toBe(updatePayload.description);

      const tree = await getConversationTree(conversationId);
      expect(tree.conversation_id).toBe(conversationId);
      expect(tree.total_messages).toBe(1);
      expect(tree.messages).toHaveLength(1);
      const [rootMessage] = tree.messages;
      expect(rootMessage.role).toBe("root");
      expect(rootMessage.depth).toBe(1);
      expect(rootMessage.lineage).toHaveLength(1);
      expect(rootMessage.parent_message_id).toBeNull();
    } finally {
      await deleteConversationIfExists(conversationId);
    }
  });

  it("maintains tree-based message lineage across diverse content types", async () => {
    const owner = "ts-lineage-owner";
    const conversation = await createConversation({
      title: `Lineage ${uniqueName("conv")}`,
      created_by: owner
    });
    const conversationId = conversation.conversation_id;

    try {
      const rootMessage = await getRootMessage(conversationId);

      const humanMessage = await appendMessage(conversationId, {
        parent_message_id: rootMessage.message_id,
        role: "human",
        content: {
          type: "text",
          text: "Human message exploring a new idea."
        },
        content_metadata: { tone: "curious" },
        created_by: owner
      });

      const imageMessage = await appendMessage(conversationId, {
        parent_message_id: rootMessage.message_id,
        role: "assistant",
        content: {
          type: "image",
          image_url: "https://example.com/image.png",
          width: 512,
          height: 512,
          mime_type: "image/png"
        },
        created_by: "ts-image-bot"
      });

      const toolCallId = uniqueName("tool-call");
      const toolCallMessage = await appendMessage(conversationId, {
        parent_message_id: humanMessage.message_id,
        role: "tool",
        content: {
          type: "tool_call",
          tool_name: "search",
          tool_call_id: toolCallId,
          arguments: {
            query: "integration testing best practices"
          }
        },
        created_by: "ts-tool-executor",
        content_metadata: { latency_ms: "134" }
      });

      const toolResultMessage = await appendMessage(conversationId, {
        parent_message_id: toolCallMessage.message_id,
        role: "tool",
        content: {
          type: "tool_result",
          tool_call_id: toolCallId,
          success: true,
          result: {
            summary: "Integration tests validate behavior across system boundaries."
          }
        },
        created_by: "ts-tool-executor"
      });

      const imageBatchMessage = await appendMessage(conversationId, {
        parent_message_id: humanMessage.message_id,
        role: "assistant",
        content: {
          type: "image_batch",
          images: [
            {
              image_url: "https://example.com/generated-1.png",
              prompt: "A branching conversation tree in neon colors"
            },
            {
              image_url: "https://example.com/generated-2.png",
              model: "stable-diffusion"
            }
          ]
        },
        created_by: "ts-image-bot"
      });

      const fetchedToolResult = await getMessage(
        conversationId,
        toolResultMessage.message_id
      );
      expect(fetchedToolResult.depth).toBe(fetchedToolResult.lineage.length);
      expect(fetchedToolResult.content.type).toBe("tool_result");
      expect(fetchedToolResult.lineage).toEqual([
        rootMessage.message_id,
        humanMessage.message_id,
        toolCallMessage.message_id,
        toolResultMessage.message_id
      ]);

      const lineagePath = await getMessageLineage(
        conversationId,
        toolResultMessage.message_id
      );
      expect(lineagePath.map((msg) => msg.message_id)).toEqual(
        fetchedToolResult.lineage
      );

      const rootChildren = await getMessageChildren(
        conversationId,
        rootMessage.message_id
      );
      const rootChildIds = rootChildren.map((msg) => msg.message_id);
      expect(rootChildIds).toContain(humanMessage.message_id);
      expect(rootChildIds).toContain(imageMessage.message_id);

      const tree = await getConversationTree(conversationId);
      expect(tree.total_messages).toBe(6);
      expect(tree.messages.map((msg) => msg.content.type)).toEqual(
        expect.arrayContaining([
          "metadata",
          "text",
          "image",
          "tool_call",
          "tool_result",
          "image_batch"
        ])
      );
    } finally {
      await deleteConversationIfExists(conversationId);
    }
  });

  it("supports branch lifecycle and automatic leaf updates", async () => {
    const owner = "ts-branch-owner";
    const conversation = await createConversation({
      title: `Branch ${uniqueName("conv")}`,
      created_by: owner
    });
    const conversationId = conversation.conversation_id;

    try {
      const rootMessage = await getRootMessage(conversationId);
      const seedMessage = await appendMessage(conversationId, {
        parent_message_id: rootMessage.message_id,
        role: "human",
        content: {
          type: "text",
          text: "Seed message for branch creation."
        },
        created_by: owner
      });

      const branch = await createBranch(conversationId, {
        branch_name: `Ideation ${uniqueName("branch")}`,
        leaf_message_id: seedMessage.message_id,
        created_by: owner
      });
      expect(branch.leaf_message_id).toBe(seedMessage.message_id);

      const branchMessages = await getBranchMessages(
        conversationId,
        branch.branch_id
      );
      expect(branchMessages.map((msg) => msg.message_id)).toEqual([
        rootMessage.message_id,
        seedMessage.message_id
      ]);

      const followUp = await appendMessage(conversationId, {
        parent_message_id: seedMessage.message_id,
        role: "assistant",
        content: {
          type: "text",
          text: "Branch follow-up extending the discussion."
        },
        created_by: "ts-branch-bot",
        branch_id: branch.branch_id
      });

      const updatedBranch = await getBranch(
        conversationId,
        branch.branch_id
      );
      expect(updatedBranch.leaf_message_id).toBe(followUp.message_id);

      const renameResponse = await httpClient.put<BranchResponse>(
        `/api/v1/conversations/${conversationId}/branches/${branch.branch_id}`,
        {
          branch_name: `Renamed ${uniqueName("branch")}`
        }
      );
      expect(renameResponse.status).toBe(200);
      expect(renameResponse.data.branch_name).toContain("Renamed");

      const deleteResponse = await httpClient.delete(
        `/api/v1/conversations/${conversationId}/branches/${branch.branch_id}`
      );
      expect(deleteResponse.status).toBe(200);
    } finally {
      await deleteConversationIfExists(conversationId);
    }
  });

  it("forks conversations, branches, and messages with provenance metadata", async () => {
    const owner = "ts-fork-owner";
    const conversation = await createConversation({
      title: `Fork Source ${uniqueName("conv")}`,
      created_by: owner
    });
    const conversationId = conversation.conversation_id;
    const forkedConversationIds: string[] = [];

    try {
      const rootMessage = await getRootMessage(conversationId);
      const firstChild = await appendMessage(conversationId, {
        parent_message_id: rootMessage.message_id,
        role: "human",
        content: {
          type: "text",
          text: "Original conversation content."
        },
        created_by: owner
      });

      const branchLeaf = await appendMessage(conversationId, {
        parent_message_id: firstChild.message_id,
        role: "assistant",
        content: {
          type: "text",
          text: "Assistant suggestion within the source tree."
        },
        created_by: "ts-fork-helper"
      });

      const branch = await createBranch(conversationId, {
        branch_name: `Forkable ${uniqueName("branch")}`,
        leaf_message_id: branchLeaf.message_id,
        created_by: owner
      });

      const forkConversationResponse = await httpClient.post<ConversationResponse>(
        `/api/v1/conversations/${conversationId}/fork`,
        {
          title: `Forked Conversation ${uniqueName("conv")}`,
          created_by: "ts-fork-user"
        }
      );
      expect(forkConversationResponse.status).toBe(200);
      expect(forkConversationResponse.data.fork_from_conversation_id).toBe(
        conversationId
      );
      expect(forkConversationResponse.data.fork_from_message_id).toBeNull();
      forkedConversationIds.push(forkConversationResponse.data.conversation_id);

      const forkBranchResponse = await httpClient.post<ConversationResponse>(
        `/api/v1/conversations/${conversationId}/branches/${branch.branch_id}/fork`,
        {
          title: `Forked Branch ${uniqueName("conv")}`,
          created_by: "ts-fork-user"
        }
      );
      expect(forkBranchResponse.status).toBe(200);
      expect(forkBranchResponse.data.fork_from_conversation_id).toBe(
        conversationId
      );
      expect(forkBranchResponse.data.fork_from_message_id).toBe(
        branchLeaf.message_id
      );
      forkedConversationIds.push(forkBranchResponse.data.conversation_id);

      const forkFromMessageResponse = await httpClient.post<ConversationResponse>(
        `/api/v1/conversations/${conversationId}/messages/${firstChild.message_id}/fork`,
        {
          title: `Forked Message ${uniqueName("conv")}`,
          created_by: "ts-fork-user"
        }
      );
      expect(forkFromMessageResponse.status).toBe(200);
      expect(forkFromMessageResponse.data.fork_from_conversation_id).toBe(
        conversationId
      );
      expect(forkFromMessageResponse.data.fork_from_message_id).toBe(
        firstChild.message_id
      );
      forkedConversationIds.push(forkFromMessageResponse.data.conversation_id);
    } finally {
      for (const forkId of forkedConversationIds) {
        await deleteConversationIfExists(forkId);
      }
      await deleteConversationIfExists(conversationId);
    }
  });

  it("shares conversations with fine-grained permissions and revokes access", async () => {
    const owner = "ts-share-owner";
    const conversation = await createConversation({
      title: `Sharing ${uniqueName("conv")}`,
      created_by: owner
    });
    const conversationId = conversation.conversation_id;
    const shareUsers = [
      { userId: `user-read-${uniqueName("usr")}`, permission: "read" as const },
      { userId: `user-branch-${uniqueName("usr")}`, permission: "branch" as const },
      { userId: `user-fork-${uniqueName("usr")}`, permission: "fork" as const }
    ];

    try {
      for (const { userId, permission } of shareUsers) {
        const share = await shareConversationWithUser(conversationId, {
          shared_with: userId,
          permission,
          shared_by: owner
        });
        expect(share.permission).toBe(permission);
        expect(share.shared_with).toBe(userId);
      }

      const sharesResponse = await httpClient.get<ShareResponse[]>(
        `/api/v1/conversations/${conversationId}/shares`
      );
      expect(sharesResponse.status).toBe(200);
      expect(sharesResponse.data).toHaveLength(3);

      const permissionMap = new Map(
        sharesResponse.data.map((share) => [share.shared_with, share.permission])
      );
      for (const { userId, permission } of shareUsers) {
        expect(permissionMap.get(userId)).toBe(permission);
      }

      await revokeShareStrict(conversationId, shareUsers[1].userId);

      const postRevokeResponse = await httpClient.get<ShareResponse[]>(
        `/api/v1/conversations/${conversationId}/shares`
      );
      expect(postRevokeResponse.status).toBe(200);
      expect(postRevokeResponse.data).toHaveLength(2);
      expect(
        postRevokeResponse.data.map((share) => share.shared_with)
      ).not.toContain(shareUsers[1].userId);

      // Clean up any remaining shares to avoid cross-test interference.
      await revokeShareIfExists(conversationId, shareUsers[0].userId);
      await revokeShareIfExists(conversationId, shareUsers[2].userId);
    } finally {
      await deleteConversationIfExists(conversationId);
    }
  });
});
