import { invoke } from "@tauri-apps/api/core";

/** Mirrors `demysto_core::Status`. */
export type Status = {
  version: string;
  config_dir: string;
};

export function status(): Promise<Status> {
  return invoke<Status>("status");
}
