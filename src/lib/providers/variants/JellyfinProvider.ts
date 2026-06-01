import { Api as JellyfinApi, Jellyfin } from "@jellyfin/sdk";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { error } from "@sveltejs/kit";

import type { Provider } from "./Provider";
import { device, uuid } from "../shared";
import { providerItems } from "$lib/db/schema/schema";
import { db } from "$lib/db/database";
import { providerManager } from "../ProviderManager";

export class JellyfinProvider implements Provider {
  readonly client: Jellyfin;
  readonly type = "JellyfinProvider";

  private _api?: JellyfinApi;
  private _authenticated = false;

  serverId: string;
  url: string;
  userId: string;

  constructor(
    serverId?: string,
    url?: string,
    userId?: string,
    accessToken?: string,
  ) {
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

    this.serverId = serverId ? serverId : "";
    this.userId = userId ? userId : "";

    this.url = "";
    if (url) {
      this.createApi(url, accessToken);
    }
  }

  public createApi(url: string, accessToken?: string) {
    this.setURL(url);

    if (accessToken) {
      this._api = this.client.createApi(url, accessToken);
      this._authenticated = true;
      return;
    }

    this._api = this.client.createApi(url, accessToken);
  }

  async authApiWithPw(uname: string, psw: string) {
    if (this.authStatus()) {
      return;
    }

    let auth = await getUserApi(this._api || error(404)).authenticateUserByName(
      {
        authenticateUserByName: {
          Username: uname,
          Pw: psw,
        },
      },
    );

    if (auth.data.AccessToken && auth.data.ServerId) {
      this.setServerId(auth.data.ServerId);
      this.setUserId(auth.data.User?.Id!);

      if (!providerManager.existsProviderWith(this.userId)) {

        localStorage.setItem(
          `${this.getServerId()}Token`,
          auth.data.AccessToken,
        );
        this.addToDb();
        this._authenticated = true;

        console.log("hello")

        providerManager.addProvider(this);
        return;
      }

      console.error("user already exists, not adding connection");
    }
  }

  async addToDb(): Promise<void> {
    if (this._api && this.url !== "" && this.serverId !== "") {
      await db
        .insert(providerItems)
        .values(this)
        .catch((e) => {
          console.error("Failed to add to DB, reason unknown");
          throw e;
        });
    }
  }

  public removeConnection() {
    localStorage.removeItem(`${this.serverId}Token`);
    providerManager.removeProvider(this);

    this.createApi(this.url);
    this._authenticated = false;
  }

  public getApi(): JellyfinApi | undefined {
    return this._api;
  }

  public getServerId(): string {
    return this.serverId;
  }

  public getUserId(): string {
    return this.userId;
  }

  private setServerId(serverId: string) {
    this.serverId = serverId;
  }

  private setURL(url: string) {
    this.url = url;
  }

  private setUserId(userId: string) {
    this.userId = userId;
  }

  public authStatus(): boolean {
    console.log(this._authenticated)
    return this._authenticated;
  }
}
