import { Jellyfin } from "@jellyfin/sdk";
import { Api as JellyfinApi } from "@jellyfin/sdk";
import { device, uuid } from "../shared";
import type { Provider } from "./Provider";

export class JellyfinProvider implements Provider {
  readonly client: Jellyfin;
  private _serverID?: string;
  private _api?: JellyfinApi;

  constructor(url?: string, token?: string) {
    this.client = new Jellyfin({
      clientInfo: {
        name: import.meta.env.VITE_JOURNEY_NAME,
        version: import.meta.env.VITE_JOURNEY_VERSION,
      },
      deviceInfo: {
        name: device,
        id: uuid,
      },
    });

    if (url) {
      this.createApi(url, token);
    }
  }

  public createApi(url: string, token?: string): JellyfinApi {
    console.log(`url: ${url}, token: ${token}`);
    this._api = this.client.createApi(url, token);
    return this._api;
  }

  public getApi(): JellyfinApi | undefined {
    return this._api;
  }

  public getServerID(): string | undefined {
    return this._serverID;
  }
}
