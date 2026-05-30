import { v4 as uuidv4 } from "uuid";
import { arch, hostname, platform, version } from "@tauri-apps/plugin-os";

export const uuid = uuidv4();
export const device =
  hostname() + "-" + platform() + "-" + arch() + "" + version();
