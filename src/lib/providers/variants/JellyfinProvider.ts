import { Api as JellyfinApi, Jellyfin } from "@jellyfin/sdk";
import { device, uuid } from "../shared";
import type { Provider } from "./Provider";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { error } from "@sveltejs/kit";

export class JellyfinProvider implements Provider {
  readonly client: Jellyfin;
  readonly type = "JellyfinProvider";

  private _api?: JellyfinApi;
  private _authenticated = false;

  id: string;
  url: string;

  constructor(id?: string, url?: string, accessToken?: string) {
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
    } else {
      this.id = "";
    }

    if (url) {
      this.url = url;
      this.createApi(url, accessToken);
    } else {
      this.url = "";
    }

    this.authStatus();
  }

  public createApi(url: string, accessToken?: string): JellyfinApi {
    this._api = this.client.createApi(url, accessToken);

    if (!this._api.accessToken) {
      const tmpUname = localStorage.getItem(`${this.getID()}Uname`);
      const tmpPsw = localStorage.getItem(`${this.getID()}Psw`);

      if (tmpUname && tmpPsw) {
        this._api = this.authApiWithPw(tmpUname, tmpPsw);
      }
    }
    return this._api;
  }

  public authApiWithPw(uname: string, psw: string): JellyfinApi {
    if (this.authStatus()) {
      return this._api || error(404);
    }

    getUserApi(this._api || error(404))
      .authenticateUserByName({
        authenticateUserByName: {
          Username: uname,
          Pw: psw,
        },
      })
      .then()
      .catch(() => error(404));

    return this._api || error(404);
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

  public authStatus(): boolean {
    if (this._api?.accessToken) {
      this._authenticated = true;
    }

    return this._authenticated;
  }
}
