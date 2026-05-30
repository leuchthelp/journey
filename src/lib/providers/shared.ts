import { v7 as uuidv7 } from "uuid";
import { arch, hostname, platform, version } from "@tauri-apps/plugin-os";

export const uuid = uuidv7();
export const device =
  (await hostname()) + "-" + platform() + "-" + arch() + "-" + version();
