import { Api as JellyfinApi, Jellyfin } from "@jellyfin/sdk";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { getItemsApi } from "@jellyfin/sdk/lib/utils/api/items-api";
import { getImageApi } from "@jellyfin/sdk/lib/utils/api/image-api";
import {
  BaseItemKind,
  type BaseItemDto,
} from "@jellyfin/sdk/lib/generated-client/models";
import { error } from "@sveltejs/kit";

import type { Provider } from "./Provider.ts";
import { device, uuid } from "../shared.ts";
import {
  providerItems,
  type ContentItem,
  type ImageItem,
} from "$lib/db/schema/schema.ts";
import { providerManager } from "../ProviderManager.ts";
import { mapJellyfinOptions } from ".";
import type { MediaItem, ParentItem } from "$lib/db/relations.ts";
import { db } from "$lib/db/database.ts";
import { insertMediaItem } from "$lib/db/transactions.ts";

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

    const auth = await getUserApi(
      this._api || error(404),
    ).authenticateUserByName({
      authenticateUserByName: {
        Username: uname,
        Pw: psw,
      },
    });

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
        this.indexFiles();
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

  async indexFiles() {
    if (this._api) {
      const api = this._api;
      const batchsize = 100;

      await this.getByType(api, BaseItemKind.MusicAlbum, batchsize, this);
      await this.getByType(api, BaseItemKind.MusicArtist, batchsize, this);
      await this.getByType(api, BaseItemKind.Audio, batchsize, this);
    }
  }

  private getChildren(
    api: JellyfinApi,
    itemTypes: BaseItemKind[],
    provider: JellyfinProvider,
    limit?: number,
  ) {
    return getItemsApi(api).getItems({
      userId: provider.userId,
      includeItemTypes: itemTypes,
      recursive: true,
      limit: limit,
    });
  }

  private async getByType(
    api: JellyfinApi,
    type: BaseItemKind,
    batchsize: number,
    provider: JellyfinProvider,
  ) {
    const res = await this.getChildren(api, [type], this, batchsize);
    const newItems = res.data.Items;

    let run = true;
    let offset = 0;
    if (newItems) {
      do {
        const tmp = newItems.flat().slice(offset, offset + batchsize);
        offset += batchsize;

        const newMediaItemPromises: Promise<MediaItem>[] = [];
        for (const item of tmp) {
          newMediaItemPromises.push(
            provider.generateMediaItem(api, item, provider),
          );
        }

        const toDbPromises: Promise<void>[] = [];
        for (const item of await Promise.all(newMediaItemPromises)) {
          toDbPromises.push(insertMediaItem(item));
        }

        await Promise.all(toDbPromises);

        if (tmp.length < batchsize) run = false;
      } while (run);
    }
  }

  private async generateMediaItem(
    api: JellyfinApi,
    item: BaseItemDto,
    provider: JellyfinProvider,
  ) {
    if (!item.Type) throw Error;

    const mediaItem = mapJellyfinOptions.get(item.Type);
    if (!mediaItem) throw Error;

    const init = new mediaItem();

    const images = provider.getImageInfoObject(api, item, provider);

    const userData = item.UserData;
    if (userData) {
      const key = userData.Key ?? "";

      const regex =
        /[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}/;
      const musicbrainzId = key.match(regex)?.toString();
      init.uuid = musicbrainzId ?? item.Id + "-tmp";
    }

    init.providers.push(provider);
    init.content.push(...provider.getItemContent(item, init));
    if (item.Id)
      init.original.push({
        uuid: item.Id,
        parentId: init.uuid,
        serverId: provider.serverId,
      });

    init.parents.push(...provider.getParents(item));
    init.images.push(...(await images));
    return init;
  }

  private async getImageInfoObject(
    api: JellyfinApi,
    item: BaseItemDto,
    provider: JellyfinProvider,
  ) {
    const images: ImageItem[] = [];

    if (item.ImageTags && Object.keys(item.ImageTags).length !== 0) {
      const info = await getImageApi(api).getItemImageInfos({
        itemId: item.Id!,
      });

      for (const entry of info.data) {
        images.push({
          serverId: provider.serverId,
          url: `${info.config.url}/${entry.ImageType}`,
          type: entry.ImageType ?? "",
        });
      }
    }
    return images;
  }

  private getItemContent(item: BaseItemDto, init: MediaItem) {
    const info: ContentItem[] = [];

    if (item.Name)
      info.push({ type: "Name", description: item.Name, parentId: init.uuid });
    if (item.Album)
      info.push({
        type: "Album",
        description: item.Album,
        parentId: init.uuid,
      });
    if (item.AlbumArtist)
      info.push({
        type: "Artists",
        description: item.AlbumArtist,
        parentId: init.uuid,
      });
    if (item.Container)
      info.push({
        type: "Container",
        description: item.Container,
        parentId: init.uuid,
      });
    if (item.PremiereDate)
      info.push({
        type: "Release-Date",
        description: item.PremiereDate,
        parentId: init.uuid,
      });

    return info;
  }

  private getParents(item: BaseItemDto) {
    const parents: ParentItem[] = [];

    if (item.AlbumArtists) {
      for (const artist of item.AlbumArtists)
        if (artist.Id) parents.push({ uuid: artist.Id });
    }

    if (item.AlbumId) parents.push({ uuid: item.AlbumId });

    return parents;
  }
}
