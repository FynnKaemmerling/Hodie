import { emit } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { api, type VaultInfo } from "./api";
import { VAULT_CHANGED } from "./api/events";

/** Asks for a folder; null if the dialog was cancelled. */
export async function chooseVaultFolder(
  title: string,
  current: string | null,
): Promise<string | null> {
  const picked = await open({ directory: true, title, defaultPath: current ?? undefined });
  return typeof picked === "string" ? picked : null;
}

/**
 * Opens `path` as the vault and lets every window start over with it.
 * Rejects with an error code if the folder cannot be used.
 */
export async function switchVault(path: string): Promise<void> {
  await api.openVault(path);
  await emit(VAULT_CHANGED);
}

/** Tries the configured folder again; reloads the windows once it opens. */
export async function retryVault(): Promise<VaultInfo> {
  const info = await api.reopenVault();
  if (info.error === null) await emit(VAULT_CHANGED);
  return info;
}
