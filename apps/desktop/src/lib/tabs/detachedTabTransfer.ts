import { serializeOpenTabs, type SavedOpenTab } from "@/lib/app/openTabsPersistence";
import type { QueryTab } from "@/types/database";

export const MAX_DETACHED_TAB_TRANSFER_BYTES = 1024 * 1024;

type DetachedTabEphemeralState = Pick<
  QueryTab,
  "editorViewport" | "editorSelection" | "autoCommit" | "hbaseCreateTableOnOpen" | "nacosTargetDataId" | "nacosTargetGroup" | "nacosTargetKeyword" | "nacosTargetRequestId" | "structureInitialTab" | "structureInitialTabRequestId" | "structureInitialTarget" | "mongoBucket" | "tableInfoTab"
>;

/**
 * Explicit allow-list for the WebView hand-off. Result rows, result runs,
 * execution/session IDs and result-cache identities must never cross IPC.
 */
export type DetachedTabDescriptor = Omit<SavedOpenTab, "resultBaseSql" | "resultSortedSql" | "resultSortColumn" | "resultSortColumnIndex" | "resultSortDirection" | "resultSortMode" | "resultEvicted" | "resultCacheKey" | "resultRuns" | "activeResultRunId"> &
  DetachedTabEphemeralState & {
    mode: QueryTab["mode"];
  };

export function lightweightDetachedTab(tab: QueryTab): DetachedTabDescriptor {
  const persisted = serializeOpenTabs([tab])[0];
  if (!persisted) throw new Error("Detached tab persistence payload is missing");
  const {
    resultBaseSql: _resultBaseSql,
    resultSortedSql: _resultSortedSql,
    resultSortColumn: _resultSortColumn,
    resultSortColumnIndex: _resultSortColumnIndex,
    resultSortDirection: _resultSortDirection,
    resultSortMode: _resultSortMode,
    resultEvicted: _resultEvicted,
    resultCacheKey: _resultCacheKey,
    resultRuns: _resultRuns,
    activeResultRunId: _activeResultRunId,
    ...lightweight
  } = persisted;

  return {
    ...lightweight,
    mode: tab.mode,
    // Unlike restart persistence, a live move must preserve the current editor
    // text even when it is backed by a saved SQL entry or external file.
    sql: tab.sql,
    originalSql: tab.originalSql,
    editorViewport: tab.editorViewport,
    editorSelection: tab.editorSelection,
    autoCommit: tab.autoCommit,
    hbaseCreateTableOnOpen: tab.hbaseCreateTableOnOpen,
    nacosTargetDataId: tab.nacosTargetDataId,
    nacosTargetGroup: tab.nacosTargetGroup,
    nacosTargetKeyword: tab.nacosTargetKeyword,
    nacosTargetRequestId: tab.nacosTargetRequestId,
    structureInitialTab: tab.structureInitialTab,
    structureInitialTabRequestId: tab.structureInitialTabRequestId,
    structureInitialTarget: tab.structureInitialTarget,
    mongoBucket: tab.mongoBucket,
    tableInfoTab: tab.tableInfoTab,
  };
}

export function restoreDetachedTab(descriptor: DetachedTabDescriptor): QueryTab {
  return {
    ...descriptor,
    isExecuting: false,
    isCancelling: false,
    isExplaining: false,
    resultTotalRowCountLoading: false,
    result: undefined,
    results: undefined,
    resultRuns: undefined,
    activeResultRunId: undefined,
    resultSessionId: undefined,
    resultCacheKey: undefined,
    resultCacheState: undefined,
    resultEvicted: undefined,
    executionId: undefined,
    explainExecutionId: undefined,
    explainClientSessionId: undefined,
    txnSessionId: undefined,
  };
}

function minimumJsonBytes(value: unknown, seen = new WeakSet<object>(), limit = Number.POSITIVE_INFINITY): number {
  if (value === null || value === undefined) return 4;
  // Every UTF-16 code unit occupies at least one UTF-8 byte after JSON
  // serialization. This lower bound catches giant values without constructing
  // a second giant string, while the exact pass below handles escapes/Unicode.
  if (typeof value === "string") return 2 + value.length;
  if (typeof value === "number") return 1;
  if (typeof value === "boolean") return 5;
  if (typeof value !== "object") throw new Error(`Detached tab payload contains unsupported ${typeof value}`);
  if (seen.has(value)) throw new Error("Detached tab payload is not serializable: circular reference");
  seen.add(value);
  let bytes = 2;
  const entries = Array.isArray(value) ? value.map((entry, index) => [String(index), entry] as const) : Object.entries(value);
  for (const [key, entry] of entries) {
    bytes += 1;
    if (!Array.isArray(value)) bytes += 3 + key.length;
    bytes += minimumJsonBytes(entry, seen, limit - bytes);
    if (bytes > limit) break;
  }
  seen.delete(value);
  return bytes;
}

export function detachedTransferPayloadBytes(payload: unknown): number {
  let serialized: string;
  try {
    serialized = JSON.stringify(payload);
  } catch (error) {
    throw new Error(`Detached tab payload is not serializable: ${error instanceof Error ? error.message : String(error)}`);
  }
  return new TextEncoder().encode(serialized).byteLength;
}

export function assertDetachedTransferPayloadBounded(payload: unknown, maxBytes = MAX_DETACHED_TAB_TRANSFER_BYTES): number {
  // Reject obviously large payloads without first allocating another giant JSON
  // string. The exact serialization only runs after this allocation-safe preflight.
  if (minimumJsonBytes(payload, new WeakSet<object>(), maxBytes) > maxBytes) {
    throw new Error(`Detached tab payload exceeds the ${Math.floor(maxBytes / (1024 * 1024))} MiB transfer limit`);
  }
  const bytes = detachedTransferPayloadBytes(payload);
  if (bytes > maxBytes) {
    throw new Error(`Detached tab payload exceeds the ${Math.floor(maxBytes / (1024 * 1024))} MiB transfer limit`);
  }
  return bytes;
}
