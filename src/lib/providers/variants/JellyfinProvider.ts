import { Api as JellyfinApi, Jellyfin } from "@jellyfin/sdk";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { error } from "@sveltejs/kit";

import type { Provider } from "./Provider";
import { device, uuid } from "../shared";
import { providerItems } from "$lib/db/schema/schema";
import { db } from "$lib/db/database";

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

    this.id = "";
    this.url = "";

    if (id) {
      this.id = id;
    }

    if (url) {
      this.createApi(url, accessToken);
    }
  }

  public createApi(url: string, accessToken?: string) {
    this.setURL(url);

    if (accessToken) {
      this._api = this.client.createApi(url, accessToken);
      this.authStatus();
      return;
    }
  }

  public authApiWithPw(uname: string, psw: string) {
    if (this.authStatus()) {
      return;
    }

    getUserApi(this._api || error(404))
      .authenticateUserByName({
        authenticateUserByName: {
          Username: uname,
          Pw: psw,
        },
      })
      .then((auth) => {
        if (auth.data.AccessToken && auth.data.ServerId) {
          this.setID(auth.data.ServerId);
          localStorage.setItem(`${this.getID()}Token`, auth.data.AccessToken);
          localStorage.setItem(`${this.getID()}Uname`, uname);
          localStorage.setItem(`${this.getID()}Psw`, psw);

          this.addToDB();
        }
      });
  }

  async addToDB(): Promise<void> {
    if (this._api && this.url !== "" && this.id !== "") {
      await db
        .insert(providerItems)
        .values(this)
        .onConflictDoNothing()
        .catch((e) => {
          console.error("Failed to add to DB, reason unknown");
          throw e;
        });
    }
  }

  public removeConnection() {
    localStorage.removeItem(`${this.id}Psw`);
    localStorage.removeItem(`${this.id}Uname`);
    localStorage.removeItem(`${this.id}Token`);

    this.createApi(this.url);
  }

  public getApi(): JellyfinApi | undefined {
    return this._api;
  }

  public getID(): string | undefined {
    return this.id;
  }

  private setID(id: string) {
    this.id = id;
  }

  private setURL(url: string) {
    this.url = url;
  }

  public authStatus(): boolean {
    if (this._api?.accessToken) {
      this._authenticated = true;
      return this._authenticated;
    }

    this._authenticated = false;
    return this._authenticated;
  }
}
