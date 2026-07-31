import { describe, expect, it } from "vitest";
import { applyDetachedConnectionMutation, buildDetachedConnectionMutation } from "@/lib/connection/connectionWindowSync";
import type { ConnectionConfig } from "@/types/database";

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

describe("detached connection ownership", () => {
  it("merges a child edit without deleting connections added in main", () => {
    const original = connection("primary", "old-host");
    const addedInMain = connection("new-main", "main-host");
    const childEdit = { ...original, host: "child-host" };
    const mutation = buildDetachedConnectionMutation([original], [childEdit]);

    const reconciled = applyDetachedConnectionMutation([original, addedInMain], mutation);

    expect(reconciled).toEqual([childEdit, addedInMain]);
  });

  it("rejects a stale child edit instead of overwriting a newer main edit", () => {
    const original = connection("primary", "old-host");
    const editedInMain = { ...original, host: "main-host" };
    const editedInChild = { ...original, host: "child-host" };
    const mutation = buildDetachedConnectionMutation([original], [editedInChild]);

    expect(() => applyDetachedConnectionMutation([editedInMain], mutation)).toThrow("Connection changed in the main window: primary");
  });

  it("rejects stale removals and additions that would resurrect an existing id", () => {
    const original = connection("primary", "old-host");
    const removal = buildDetachedConnectionMutation([original], []);
    expect(() => applyDetachedConnectionMutation([], removal)).toThrow("Connection changed in the main window: primary");

    const addition = buildDetachedConnectionMutation([], [original]);
    expect(() => applyDetachedConnectionMutation([connection("primary", "main-host")], addition)).toThrow("Connection changed in the main window: primary");
  });

  it("preserves newer runtime database metadata during a user edit", () => {
    const original = connection("primary", "old-host");
    const childEdit = { ...original, note: "updated note" };
    const authoritative = {
      ...original,
      database_info: {
        productName: "PostgreSQL",
        productVersion: "18.1",
      },
    };
    const mutation = buildDetachedConnectionMutation([original], [childEdit]);

    expect(applyDetachedConnectionMutation([authoritative], mutation)[0]).toMatchObject({
      note: "updated note",
      database_info: authoritative.database_info,
    });
  });
});
