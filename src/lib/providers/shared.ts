import { v7 as uuidv7 } from "uuid";
import { isTauri } from "@tauri-apps/api/core";
import { arch, hostname, platform, version } from "@tauri-apps/plugin-os";

async function tauriDevice() {
  return (await hostname()) + "-" + platform() + "-" + arch() + "-" + version();
}

const uuid = uuidv7();
const device: string = isTauri() ? await tauriDevice() : navigator.platform;

export { uuid, device };
