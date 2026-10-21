import { invoke } from "@tauri-apps/api/core";

export function openBrowser(): Promise<void> {
  return invoke("open_browser");
}

export function openFileExplorer(): Promise<void> {
  return invoke("open_file_explorer");
}
