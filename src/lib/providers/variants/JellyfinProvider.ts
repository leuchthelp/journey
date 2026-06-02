import { Api as JellyfinApi, Jellyfin } from "@jellyfin/sdk";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { getLibraryApi } from "@jellyfin/sdk/lib/utils/api/library-api";
import { getItemsApi } from "@jellyfin/sdk/lib/utils/api/items-api";
import {
  BaseItemKind,
  type BaseItemDto,
} from "@jellyfin/sdk/lib/generated-client/models";
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
        name: "journey",
        version: "0.1.0",
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
      this.setAuthStatus(true);
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

    if (auth.data.AccessToken && auth.data.ServerId && auth.data.User?.Id) {
      this.setServerId(auth.data.ServerId);
      this.setUserId(auth.data.User?.Id!);

      if (providerManager.existsProviderWith(this.userId)) {
        if (!providerManager.getProviderByUserId(this.userId).authStatus()) {
          localStorage.setItem(
            `${this.getServerId()}Token`,
            auth.data.AccessToken,
          );
          this.setAuthStatus(true);
          return;
        }
      }

      if (!providerManager.existsProviderWith(this.userId)) {
        localStorage.setItem(
          `${this.getServerId()}Token`,
          auth.data.AccessToken,
        );
        this.addToDb();
        providerManager.addProvider(this);
        this.setAuthStatus(true);
        return;
      }

      throw Error("Connection already exists");
    }
  }

  async addToDb(): Promise<void> {
    if (this._api && this.url !== "" && this.serverId !== "") {
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
    localStorage.removeItem(`${this.serverId}Token`);
    providerManager.removeProvider(this);

    this.createApi(this.url);
    this.setAuthStatus(false);
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
    return this._authenticated;
  }

  public setAuthStatus(item: boolean) {
    this._authenticated = item;
  }

  async indexFiles(): Promise<void> {
    if (this._api) {
      let api = this._api;

      let itemType = BaseItemKind;

      let libraries = this.getLibraryItems(api);
      let promisedArtists = this.bundlePromises(
        this.getChildren,
        api,
        libraries,
        itemType.MusicArtist,
      );

      let promisedAlbums = this.bundlePromises(
        this.getChildren,
        api,
        promisedArtists,
        itemType.MusicAlbum,
      );

      let promisedSongs = this.bundlePromises(
        this.getChildren,
        api,
        promisedAlbums,
        itemType.Audio,
      );

      console.log(await promisedSongs);
    }
  }

  async bundlePromises(
    func: Function,
    api: JellyfinApi,
    items: Promise<BaseItemDto[]>,
    itemType?: BaseItemKind,
  ) {
    let pool: Promise<BaseItemDto[]>[] = [];

    for (const item of await items) {
      if (item.Id) pool.push(func(api, item.Id));
    }

    return (await Promise.all(pool))
      .flat()
      .filter((item) => item.Type === itemType);
  }

  async getLibraryItems(api: JellyfinApi) {
    return (await getLibraryApi(api).getMediaFolders()).data?.Items ?? [];
  }

  async getChildren(api: JellyfinApi, id: string) {
    return (await getItemsApi(api).getItems({ parentId: id })).data.Items ?? [];
  }
}
