import { emitTo, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { isTauriRuntime } from "@/lib/backend/tauriRuntime";
import { isDetachedTabWindow } from "@/lib/tabs/tabWindow";
import type { ConnectionConfig } from "@/types/database";

const CONNECTION_MUTATION_EVENT = "dbx-detached-connection-mutation";
const CONNECTION_MUTATION_ACK_PREFIX = "dbx-detached-connection-mutation-ack:";
const CONNECTION_MUTATION_TIMEOUT_MS = 15_000;

export interface DetachedConnectionMutation {
  upserts: Array<{
    config: ConnectionConfig;
    expected: ConnectionConfig | null;
  }>;
  removals: Array<{
    connectionId: string;
    expected: ConnectionConfig;
  }>;
}

interface DetachedConnectionMutationRequest extends DetachedConnectionMutation {
  requestId: string;
  windowLabel: string;
}

interface DetachedConnectionMutationAcknowledgement {
  requestId: string;
  ok: boolean;
  connections?: ConnectionConfig[];
  message?: string;
}

function stableValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(stableValue);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>)
      .filter(([, entry]) => entry !== undefined)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => [key, stableValue(entry)]),
  );
}

function comparableConnection(config: ConnectionConfig): unknown {
  // Runtime-discovered database metadata must not conflict with or be rolled
  // back by a user-initiated edit from another WebView.
  const { database_info: _databaseInfo, ...persistedConfig } = config;
  return stableValue(persistedConfig);
}

function connectionSnapshotsEqual(left: ConnectionConfig, right: ConnectionConfig): boolean {
  return JSON.stringify(comparableConnection(left)) === JSON.stringify(comparableConnection(right));
}

export function buildDetachedConnectionMutation(current: ConnectionConfig[], next: ConnectionConfig[]): DetachedConnectionMutation {
  const currentById = new Map(current.filter((connection) => connection.one_time !== true).map((connection) => [connection.id, connection]));
  const nextById = new Map(next.filter((connection) => connection.one_time !== true).map((connection) => [connection.id, connection]));
  const upserts: DetachedConnectionMutation["upserts"] = [];
  const removals: DetachedConnectionMutation["removals"] = [];

  for (const config of nextById.values()) {
    const expected = currentById.get(config.id) ?? null;
    if (!expected || !connectionSnapshotsEqual(expected, config)) upserts.push({ config, expected });
  }
  for (const expected of currentById.values()) {
    if (!nextById.has(expected.id)) removals.push({ connectionId: expected.id, expected });
  }
  return { upserts, removals };
}

export function applyDetachedConnectionMutation(current: ConnectionConfig[], mutation: DetachedConnectionMutation): ConnectionConfig[] {
  const currentById = new Map(current.map((connection) => [connection.id, connection]));

  // Validate every expected base before applying any change. This gives the
  // main WebView an optimistic concurrency boundary and prevents stale children
  // from resurrecting removed connections or overwriting newer edits.
  for (const change of mutation.upserts) {
    const authoritative = currentById.get(change.config.id);
    if (change.expected === null) {
      if (authoritative) throw new Error(`Connection changed in the main window: ${change.config.id}`);
    } else if (!authoritative || !connectionSnapshotsEqual(authoritative, change.expected)) {
      throw new Error(`Connection changed in the main window: ${change.config.id}`);
    }
  }
  for (const change of mutation.removals) {
    const authoritative = currentById.get(change.connectionId);
    if (!authoritative || !connectionSnapshotsEqual(authoritative, change.expected)) {
      throw new Error(`Connection changed in the main window: ${change.connectionId}`);
    }
  }

  const removedIds = new Set(mutation.removals.map((change) => change.connectionId));
  const upsertsById = new Map(mutation.upserts.map((change) => [change.config.id, change.config]));
  const reconciled = current
    .filter((connection) => !removedIds.has(connection.id))
    .map((connection) => {
      const updated = upsertsById.get(connection.id);
      if (!updated) return connection;
      upsertsById.delete(connection.id);
      return {
        ...updated,
        // Preserve newer runtime metadata maintained by the authoritative store.
        database_info: connection.database_info ?? updated.database_info,
      };
    });
  reconciled.push(...upsertsById.values());
  return reconciled;
}

function acknowledgementEventName(requestId: string): string {
  return `${CONNECTION_MUTATION_ACK_PREFIX}${requestId.replace(/[^a-zA-Z0-9_-]/g, "-")}`;
}

export async function requestMainConnectionMutation(mutation: DetachedConnectionMutation): Promise<ConnectionConfig[]> {
  if (!isDetachedTabWindow()) throw new Error("Connection mutation forwarding requires a detached window");
  if (mutation.upserts.length === 0 && mutation.removals.length === 0) return [];

  const currentWindow = getCurrentWebviewWindow();
  const requestId = crypto.randomUUID();
  const eventName = acknowledgementEventName(requestId);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let unlisten: UnlistenFn = () => {};
  let resolveAcknowledgement: (value: DetachedConnectionMutationAcknowledgement) => void = () => {};
  let rejectAcknowledgement: (error: Error) => void = () => {};
  const acknowledgement = new Promise<DetachedConnectionMutationAcknowledgement>((resolve, reject) => {
    resolveAcknowledgement = resolve;
    rejectAcknowledgement = reject;
  });
  unlisten = await currentWindow.listen<DetachedConnectionMutationAcknowledgement>(eventName, (event) => {
    if (event.payload.requestId === requestId) resolveAcknowledgement(event.payload);
  });
  timer = setTimeout(() => rejectAcknowledgement(new Error("Main window did not persist the connection change")), CONNECTION_MUTATION_TIMEOUT_MS);

  try {
    await emitTo("main", CONNECTION_MUTATION_EVENT, {
      ...mutation,
      requestId,
      windowLabel: currentWindow.label,
    } satisfies DetachedConnectionMutationRequest);
    const result = await acknowledgement;
    if (!result.ok) throw new Error(result.message || "Main window rejected the connection change");
    return result.connections ?? [];
  } finally {
    if (timer) clearTimeout(timer);
    unlisten();
  }
}

export async function listenForDetachedConnectionMutations(onMutation: (mutation: DetachedConnectionMutation) => Promise<ConnectionConfig[]>): Promise<UnlistenFn> {
  if (!isTauriRuntime()) return () => {};
  return getCurrentWebviewWindow().listen<DetachedConnectionMutationRequest>(CONNECTION_MUTATION_EVENT, async (event) => {
    const request = event.payload;
    let acknowledgement: DetachedConnectionMutationAcknowledgement;
    try {
      const connections = await onMutation({
        upserts: request.upserts,
        removals: request.removals,
      });
      acknowledgement = { requestId: request.requestId, ok: true, connections };
    } catch (error) {
      acknowledgement = {
        requestId: request.requestId,
        ok: false,
        message: error instanceof Error ? error.message : String(error),
      };
    }
    await emitTo(request.windowLabel, acknowledgementEventName(request.requestId), acknowledgement).catch(() => {});
  });
}
