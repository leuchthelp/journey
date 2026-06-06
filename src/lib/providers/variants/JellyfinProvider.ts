import { Api as JellyfinApi, Jellyfin } from "@jellyfin/sdk";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { getLibraryApi } from "@jellyfin/sdk/lib/utils/api/library-api";
import { getItemsApi } from "@jellyfin/sdk/lib/utils/api/items-api";
import { getImageApi } from "@jellyfin/sdk/lib/utils/api/image-api";
import {
  BaseItemKind,
  type BaseItemDto,
} from "@jellyfin/sdk/lib/generated-client/models";
import { error } from "@sveltejs/kit";

import type { Provider } from "./Provider";
import { device, uuid } from "../shared";
import {
  providerItems,
  type ContentItem,
  type ImageItem,
} from "../../db/schema/schema";
import { providerManager } from "../ProviderManager";
import { mapJellyfinOptions } from ".";
import type { MediaItem } from "$lib/db/relations";
import { db } from "$lib/db/database";

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

      let libraries = this.getLibraryItems(api).then((libraries) => {
        let mappedLibraries: [BaseItemDto, undefined][] = [];
        for (let i = 0; i < libraries.length; i++) {
          mappedLibraries.push([libraries[i]!, undefined]);
        }

        return mappedLibraries;
      });

      let promisedArtists = this.bundlePromises(
        this.getChildren,
        api,
        libraries,
        BaseItemKind.MusicArtist,
        this,
      );

      let promisedAlbums = this.bundlePromises(
        this.getChildren,
        api,
        promisedArtists,
        BaseItemKind.MusicAlbum,
        this,
      );

      let promisedSongs = this.bundlePromises(
        this.getChildren,
        api,
        promisedAlbums,
        BaseItemKind.Audio,
        this,
      );

      await promisedSongs;
    }
  }

  private async bundlePromises(
    func: Function,
    api: JellyfinApi,
    items: Promise<[BaseItemDto, MediaItem | undefined][]>,
    itemType: BaseItemKind,
    provider?: JellyfinProvider,
  ) {
    let pool: Promise<[BaseItemDto, MediaItem | undefined][]>[] = [];

    for (const item of await items) {
      pool.push(func(api, item, itemType, provider));
    }

    let tmp = await Promise.all(pool);
    return tmp.flat();
  }

  private async getLibraryItems(api: JellyfinApi): Promise<BaseItemDto[]> {
    return (await getLibraryApi(api).getMediaFolders()).data?.Items ?? [];
  }

  private async getChildren(
    api: JellyfinApi,
    parent: [BaseItemDto, MediaItem | undefined],
    itemType: BaseItemKind,
    provider?: JellyfinProvider,
  ): Promise<[BaseItemDto, MediaItem | undefined][]> {
    let tmp =
      (await getItemsApi(api).getItems({ parentId: parent[0].Id })).data
        .Items ?? [];
    tmp.filter((item) => item.Type === itemType);

    const mediaItemPromises: Promise<MediaItem>[] = [];
    if (provider) {
      for (const item of tmp) {
        if (mapJellyfinOptions.has(itemType)) {
          mediaItemPromises.push(
            provider.generateMediaItem(
              api,
              item,
              parent[1],
              itemType,
              provider,
            ),
          );
          continue;
        }
        console.error(
          `unhandled MediaItem type: ${itemType}, need to create matching Item first`,
        );
        throw Error;
      }
    }

    const mediaItems = await Promise.all(mediaItemPromises);
    const res: [BaseItemDto, MediaItem | undefined][] = [];

    for (let i = 0; i < tmp.length; i++) {
      res.push([tmp[i]!, mediaItems[i]]);
    }

    return res;
  }

  private async generateMediaItem(
    api: JellyfinApi,
    item: BaseItemDto,
    parent: MediaItem | undefined,
    itemType: BaseItemKind,
    provider: JellyfinProvider,
  ) {
    let mediaItem = mapJellyfinOptions.get(itemType)!;

    let init = new mediaItem();
    let images = provider.getImageInfoObject(api, item, provider);

    let userData = item.UserData;

    if (userData) {
      let i = userData.Key ?? "";

      const regex =
        /[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}/;
      let musicbrainzId = i.match(regex)?.toString();
      init.uuid = musicbrainzId ?? "";
    }

    if (parent) init.parents.push(parent);
    init.providers.push(provider);
    init.content.push(...provider.getItemContent(item));
    init.images.push(...(await images));

    return init;
  }

  private async getImageInfoObject(
    api: JellyfinApi,
    item: BaseItemDto,
    provider: JellyfinProvider,
  ) {
    let info = await getImageApi(api).getItemImageInfos({
      itemId: item.Id!,
    });

    let images: ImageItem[] = [];

    for (const entry of info.data) {
      images.push({
        serverId: provider.serverId,
        url: `${info.config.url}/${entry.ImageType}`,
        type: entry.ImageType ?? "",
      });
    }
    return images;
  }

  private getItemContent(item: BaseItemDto) {
    let info: ContentItem[] = [];

    if (item.Name) info.push({ type: "Name", description: item.Name });
    if (item.Album) info.push({ type: "Album", description: item.Album });
    if (item.AlbumArtist)
      info.push({ type: "Artists", description: item.AlbumArtist });
    if (item.Container)
      info.push({ type: "Container", description: item.Container });
    if (item.PremiereDate)
      info.push({ type: "Release-Date", description: item.PremiereDate });

    return info;
  }
}
