import { Jellyfin } from "@jellyfin/sdk";
import { Api as JellyfinApi } from "@jellyfin/sdk";
import { device, uuid } from "../shared";
import type { Provider } from "./Provider";

export class JellyfinProvider implements Provider {
  readonly client: Jellyfin;
  readonly type = "JellyfinProvider";

  private _api?: JellyfinApi;

  id!: string;
  url!: string;

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
    this._api = this.client.createApi(url, token);
    return this._api;
  }

  public getApi(): JellyfinApi | undefined {
    return this._api;
  }

  public getID(): string | undefined {
    return this.id;
  }

  public setID(id: string) {
    this.id = id;
  }

  public setURL(url: string) {
    this.url = url;
  }
}
