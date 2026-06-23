import { v7 as uuidv7 } from "uuid";

async function tauriDevice() {
  const { arch, hostname, platform, version } =
    await import("@tauri-apps/plugin-os");

  return (await hostname()) + "-" + platform() + "-" + arch() + "-" + version();
}

console.log("__TAURI__" in window)

const uuid = uuidv7();
const device: string =
  "__TAURI__" in window ? await tauriDevice() : navigator.platform;

export { uuid, device };
