import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ConnectionConfig } from "@/types/database";

const mocks = vi.hoisted(() => ({
  saveConnections: vi.fn(),
}));

vi.mock("@/lib/tabs/tabWindow", () => ({
  isDetachedTabWindow: () => false,
}));

vi.mock("@/lib/backend/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/backend/api")>()),
  saveConnections: mocks.saveConnections,
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
    driver_profile: "postgres",
    driver_label: "PostgreSQL",
    url_params: "",
    agent_java_options: [],
    attached_databases: [],
    transport_layers: [],
    show_system_schemas: false,
    connect_timeout_secs: 10,
    query_timeout_secs: 30,
    idle_timeout_secs: 60,
    keepalive_interval_secs: 30,
  };
}

describe("connectionStore persistence coordinator", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.stubGlobal("localStorage", {
      getItem: vi.fn(() => null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
    });
    mocks.saveConnections.mockReset();
  });

  it("merges concurrent edits to different connections before each full-list write", async () => {
    let releaseFirstWrite: () => void = () => {};
    const firstWriteBlocked = new Promise<void>((resolve) => {
      releaseFirstWrite = resolve;
    });
    const saved: ConnectionConfig[][] = [];
    mocks.saveConnections.mockImplementation(async (connections: ConnectionConfig[]) => {
      saved.push(JSON.parse(JSON.stringify(connections)) as ConnectionConfig[]);
      if (saved.length === 1) await firstWriteBlocked;
    });
    const { useConnectionStore } = await import("@/stores/connectionStore");
    const store = useConnectionStore();
    const first = connection("first", "first-old");
    const second = connection("second", "second-old");
    store.connections = [first, second];

    const firstUpdate = store.updateConnection({ ...first, host: "first-new" });
    await vi.waitFor(() => expect(mocks.saveConnections).toHaveBeenCalledTimes(1));
    const secondUpdate = store.updateConnection({ ...second, host: "second-new" });
    releaseFirstWrite();
    await Promise.all([firstUpdate, secondUpdate]);

    expect(saved).toHaveLength(2);
    expect(saved[1].map(({ id, host }) => ({ id, host }))).toEqual([
      { id: "first", host: "first-new" },
      { id: "second", host: "second-new" },
    ]);
    expect(store.connections.map(({ id, host }) => ({ id, host }))).toEqual([
      { id: "first", host: "first-new" },
      { id: "second", host: "second-new" },
    ]);
  });

  it("rejects a concurrent stale edit to the same connection", async () => {
    let releaseFirstWrite: () => void = () => {};
    const firstWriteBlocked = new Promise<void>((resolve) => {
      releaseFirstWrite = resolve;
    });
    mocks.saveConnections.mockImplementationOnce(async () => firstWriteBlocked);
    const { useConnectionStore } = await import("@/stores/connectionStore");
    const store = useConnectionStore();
    const original = connection("primary", "old-host");
    store.connections = [original];

    const firstUpdate = store.updateConnection({ ...original, host: "first-host" });
    await vi.waitFor(() => expect(mocks.saveConnections).toHaveBeenCalledOnce());
    const staleUpdate = store.updateConnection({ ...original, host: "stale-host" });
    releaseFirstWrite();

    await firstUpdate;
    await expect(staleUpdate).rejects.toThrow("Connection changed in the main window: primary");
    expect(store.connections[0].host).toBe("first-host");
  });
});
