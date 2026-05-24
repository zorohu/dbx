import * as api from "@/lib/api";
import { buildTableSelectSql } from "@/lib/tableSelectSql";
import { editablePrimaryKeys, usesSyntheticRowIdKey } from "@/lib/tableEditing";
import { useConnectionStore } from "@/stores/connectionStore";
import { useQueryStore } from "@/stores/queryStore";
import { useSettingsStore } from "@/stores/settingsStore";

export type NavigationTarget = {
  connectionId: string;
  database: string;
  schema?: string;
  tableName: string;
  columnName?: string;
  whereInput?: string;
};

async function openTableTarget(target: NavigationTarget) {
  const connectionStore = useConnectionStore();
  const queryStore = useQueryStore();
  const settingsStore = useSettingsStore();
  const pageLimit = settingsStore.editorSettings.pageSize;

  connectionStore.activeConnectionId = target.connectionId;
  const config = connectionStore.getConfig(target.connectionId);
  const tabTitle = target.schema ? `${target.schema}.${target.tableName}` : target.tableName;
  const tabId = queryStore.createTab(target.connectionId, target.database, tabTitle, "data", target.schema);
  queryStore.setExecuting(tabId, true);

  try {
    await connectionStore.ensureConnected(target.connectionId);
    if (!config) throw new Error("Connection config not found");
    const querySchema = target.schema || target.database;
    if (config.db_type === "neo4j") {
      const columns = await api.getColumns(target.connectionId, target.database, querySchema, target.tableName);
      const primaryKeys = editablePrimaryKeys(config.db_type, columns);
      const sql = await buildTableSelectSql({
        databaseType: config.db_type,
        schema: target.schema,
        tableName: target.tableName,
        columns: columns.map((column) => column.name),
        primaryKeys,
        whereInput: target.whereInput,
        limit: pageLimit,
      });
      queryStore.updateSql(tabId, sql);
      queryStore.setTableMeta(tabId, {
        schema: target.schema,
        tableName: target.tableName,
        columns,
        primaryKeys,
      });
      await queryStore.executeTabSql(tabId, sql);
      return;
    }
    const sql = await buildTableSelectSql({
      databaseType: config.db_type,
      schema: target.schema,
      tableName: target.tableName,
      whereInput: target.whereInput,
      limit: pageLimit,
    });
    queryStore.updateSql(tabId, sql);
    queryStore.setTableMeta(tabId, {
      schema: target.schema,
      tableName: target.tableName,
      columns: [],
      primaryKeys: [],
    });
    const columnsPromise = api.getColumns(target.connectionId, target.database, querySchema, target.tableName);
    const dataPromise = queryStore.executeTabSql(tabId, sql);
    const [columnsResult, dataResult] = await Promise.allSettled([columnsPromise, dataPromise]);
    if (columnsResult.status === "fulfilled") {
      const columns = columnsResult.value;
      const primaryKeys = editablePrimaryKeys(config.db_type, columns);
      const useRowId = usesSyntheticRowIdKey(config.db_type, primaryKeys);
      queryStore.setTableMeta(tabId, {
        schema: target.schema,
        tableName: target.tableName,
        columns,
        primaryKeys,
      });
      if (useRowId || config.db_type === "tdengine") {
        const newSql = await buildTableSelectSql({
          databaseType: config.db_type,
          schema: target.schema,
          tableName: target.tableName,
          whereInput: target.whereInput,
          primaryKeys,
          columns: columns.map((column) => column.name),
          includeRowId: true,
          limit: pageLimit,
        });
        queryStore.updateSql(tabId, newSql);
        await queryStore.executeTabSql(tabId, newSql);
      }
    }
    if (dataResult.status === "rejected") throw dataResult.reason;
    if (columnsResult.status === "rejected")
      console.error("[DBX] ERROR fetching table metadata:", columnsResult.reason);
  } catch (e: any) {
    queryStore.setErrorResult(tabId, e);
  }
}

export function useNavigationTargets(dialogs: {
  showFieldLineageDialog: { value: boolean };
  showDatabaseSearchDialog: { value: boolean };
}) {
  const connectionStore = useConnectionStore();
  const queryStore = useQueryStore();

  async function openLineageTarget(target: NavigationTarget) {
    dialogs.showFieldLineageDialog.value = false;
    await openTableTarget(target);
  }

  async function openDatabaseSearchTarget(target: NavigationTarget) {
    dialogs.showDatabaseSearchDialog.value = false;
    await openTableTarget(target);
  }

  async function onStructureEditorSaved(
    reloadData: () => Promise<void>,
    toast: (msg: string, duration?: number) => void,
    context: { connectionId: string; database: string; schema?: string; tableName: string },
  ) {
    if (!context.tableName) {
      try {
        await connectionStore.refreshObjectListTreeNode(
          context.connectionId,
          context.database,
          context.schema || undefined,
        );
      } catch {}
      return;
    }
    const activeTab = queryStore.tabs.find((t) => t.id === queryStore.activeTabId);
    if (activeTab?.mode === "data" && activeTab.tableMeta?.tableName === context.tableName) {
      try {
        const columns = await api.getColumns(
          activeTab.connectionId,
          activeTab.database,
          activeTab.tableMeta.schema || activeTab.database,
          activeTab.tableMeta.tableName,
        );
        queryStore.setTableMeta(activeTab.id, {
          ...activeTab.tableMeta,
          columns,
          primaryKeys: editablePrimaryKeys(connectionStore.getConfig(activeTab.connectionId)?.db_type, columns),
        });
        await connectionStore.refreshObjectListTreeNode(
          activeTab.connectionId,
          activeTab.database,
          activeTab.tableMeta.schema,
        );
        await reloadData();
      } catch (e: any) {
        toast(e?.message || String(e), 5000);
      }
    }
  }

  return { openLineageTarget, openDatabaseSearchTarget, onStructureEditorSaved, openTableTarget };
}
