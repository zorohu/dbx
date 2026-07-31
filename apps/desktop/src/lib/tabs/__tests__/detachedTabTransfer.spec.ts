import { describe, expect, it } from "vitest";
import { assertDetachedTransferPayloadBounded, detachedTransferPayloadBytes, lightweightDetachedTab, MAX_DETACHED_TAB_TRANSFER_BYTES, restoreDetachedTab } from "@/lib/tabs/detachedTabTransfer";
import type { QueryTab } from "@/types/database";

function queryTab(): QueryTab {
  return {
    id: "query-1",
    title: "Query",
    connectionId: "connection-1",
    database: "app",
    sql: "select 1",
    mode: "query",
    isExecuting: false,
    resultSessionId: "result-session",
    executionId: "execution-1",
    txnSessionId: "transaction-1",
    resultCacheKey: "tab:query-1:result",
    resultBaseSql: "select 1",
    resultSortedSql: "select 1 order by value",
    result: {
      columns: ["value"],
      rows: [[1], [2]],
      affected_rows: 0,
      execution_time_ms: 1,
      session_id: "result-session",
    },
    resultRuns: [
      {
        id: "run-1",
        title: "Result 1",
        sequence: 1,
        sql: "select 1",
        createdAt: 1,
        result: { columns: ["value"], rows: [[1]], affected_rows: 0, execution_time_ms: 1 },
      },
    ],
  };
}

describe("detached tab transfer payload", () => {
  it("uses an allow-list that excludes results, caches, sessions and execution state", () => {
    const source = queryTab();
    const transferred = lightweightDetachedTab(source);

    expect(transferred).toMatchObject({ id: source.id, sql: source.sql, mode: "query" });
    for (const key of ["result", "results", "resultRuns", "resultSessionId", "resultCacheKey", "resultBaseSql", "resultSortedSql", "executionId", "txnSessionId"]) {
      expect(transferred).not.toHaveProperty(key);
    }
    const restored = restoreDetachedTab(transferred);
    expect(restored).toMatchObject({ isExecuting: false, isCancelling: false, isExplaining: false });
    expect(restored.result).toBeUndefined();
    expect(restored.txnSessionId).toBeUndefined();
  });

  it("measures UTF-8 bytes and rejects payloads above the configured bound", () => {
    expect(detachedTransferPayloadBytes({ value: "数据库" })).toBeGreaterThan(JSON.stringify({ value: "数据库" }).length);
    expect(() => assertDetachedTransferPayloadBounded({ sql: "x".repeat(1024) }, 128)).toThrow("Detached tab payload exceeds");
    expect(assertDetachedTransferPayloadBounded({ sql: "select 1" }, 128)).toBeLessThanOrEqual(128);
  });

  it("does not copy a result even when its rows exceed the IPC limit", () => {
    const source = queryTab();
    source.result!.rows = [["x".repeat(MAX_DETACHED_TAB_TRANSFER_BYTES + 1)]];

    const transferred = lightweightDetachedTab(source);

    expect(detachedTransferPayloadBytes({ tab: transferred })).toBeLessThan(1024 * 1024);
    expect(() => assertDetachedTransferPayloadBounded({ tab: transferred })).not.toThrow();
  });
});
