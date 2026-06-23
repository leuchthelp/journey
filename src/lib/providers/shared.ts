import { v7 as uuidv7 } from "uuid";

async function tauriDevice() {
  const { arch, hostname, platform, version } =
    await import("@tauri-apps/plugin-os");

  return (await hostname()) + "-" + platform() + "-" + arch() + "-" + version();
}

const uuid = uuidv7();
const device: string =
  "__TAURI_INTERNALS__" in window ? await tauriDevice() : navigator.platform;

export { uuid, device };
