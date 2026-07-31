import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ConnectionConfig } from "@/types/database";

const mocks = vi.hoisted(() => ({
  requestMainConnectionMutation: vi.fn(),
  saveConnections: vi.fn(),
  deleteTabResultSnapshotsForOwner: vi.fn(async () => {}),
}));

vi.mock("@/lib/tabs/tabWindow", () => ({
  isDetachedTabWindow: () => true,
}));

vi.mock("@/lib/connection/connectionWindowSync", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/connection/connectionWindowSync")>()),
  requestMainConnectionMutation: mocks.requestMainConnectionMutation,
}));

vi.mock("@/lib/backend/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/backend/api")>()),
  saveConnections: mocks.saveConnections,
}));

vi.mock("@/lib/tabs/tabResultCache", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/tabs/tabResultCache")>()),
  deleteTabResultSnapshotsForOwner: mocks.deleteTabResultSnapshotsForOwner,
}));

function connection(id: string, host: string): ConnectionConfig {
  return {
    id,
    name: id,
    db_type: "postgres",
    host,
    port: 5432,
    username: "postgres",
    password: "",
    database: "app",
  };
}

describe("connectionStore detached persistence", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.stubGlobal("localStorage", {
      getItem: vi.fn(() => null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
    });
    mocks.requestMainConnectionMutation.mockReset();
    mocks.saveConnections.mockReset();
    mocks.deleteTabResultSnapshotsForOwner.mockClear();
  });

  it("delegates a per-connection mutation and adopts the authoritative main snapshot", async () => {
    const original = connection("primary", "old-host");
    const updated = { ...original, host: "child-host" };
    const addedInMain = connection("new-main", "main-host");
    const removedInMain = connection("removed-main", "removed-host");
    mocks.requestMainConnectionMutation.mockResolvedValue([updated, addedInMain]);
    const { useConnectionStore } = await import("@/stores/connectionStore");
    const store = useConnectionStore();
    store.connections = [original, removedInMain];
    store.connectedIds.add(removedInMain.id);

    await store.updateConnection(updated);

    expect(mocks.requestMainConnectionMutation).toHaveBeenCalledWith({
      upserts: [
        {
          config: expect.objectContaining({ id: updated.id, host: updated.host }),
          expected: original,
        },
      ],
      removals: [],
    });
    expect(mocks.saveConnections).not.toHaveBeenCalled();
    expect(store.connections.map(({ id, host }) => ({ id, host }))).toEqual([
      { id: updated.id, host: updated.host },
      { id: addedInMain.id, host: addedInMain.host },
    ]);
    expect(store.connectedIds.has(removedInMain.id)).toBe(false);
  });
});
