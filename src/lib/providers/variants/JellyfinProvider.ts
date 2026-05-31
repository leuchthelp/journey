import { Jellyfin } from "@jellyfin/sdk";
import { Api as JellyfinApi } from "@jellyfin/sdk";
import { device, uuid } from "../shared";
import type { Provider } from "./Provider";

export class JellyfinProvider implements Provider {
  readonly client: Jellyfin;
  readonly type = "JellyfinProvider";

  private _api?: JellyfinApi;

  id: string;
  url: string;

  constructor(id?: string, url?: string, token?: string) {
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

    if (id) {
      this.id = id;

      token = localStorage.getItem(`${id}Token`) || undefined;
    } else {
      this.id = "";
    }

    if (url) {
      this.url = url;
      this.createApi(url, token);
    } else {
      this.url = "";
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
