import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ writeTabResultSnapshot: vi.fn(async () => true) }));

vi.mock("@/lib/tabs/tabResultCache", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/tabs/tabResultCache")>()),
  writeTabResultSnapshot: mocks.writeTabResultSnapshot,
}));

describe("queryStore detached result payload", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mocks.writeTabResultSnapshot.mockClear();
    vi.stubGlobal("localStorage", { getItem: vi.fn(() => null), setItem: vi.fn(), removeItem: vi.fn() });
  });

  it("returns identity/editor state without persisting or transferring result rows", async () => {
    const { useQueryStore } = await import("@/stores/queryStore");
    const store = useQueryStore();
    const tabId = store.createTab("pg-1", "app", "Query", "query");
    const tab = store.tabs[0];
    tab.sql = "select * from users";
    tab.editorSelection = { anchor: 3, head: 9 };
    tab.resultSessionId = "result-session";
    tab.resultCacheKey = `tab:${tabId}:result`;
    tab.result = { columns: ["value"], rows: [[1], [2]], affected_rows: 0, execution_time_ms: 1 };

    const transferred = store.createDetachedTransferTab(tab);

    expect(transferred).toMatchObject({ id: tabId, sql: "select * from users", editorSelection: { anchor: 3, head: 9 } });
    expect(transferred).not.toHaveProperty("result");
    expect(transferred).not.toHaveProperty("resultSessionId");
    expect(transferred).not.toHaveProperty("resultCacheKey");
    expect(mocks.writeTabResultSnapshot).not.toHaveBeenCalled();
    expect(tab.result?.rows).toEqual([[1], [2]]);
  }, 10_000);
});
