import { Jellyfin } from "@jellyfin/sdk";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { getLibraryApi } from "@jellyfin/sdk/lib/utils/api/library-api";
import { getUserLibraryApi } from "@jellyfin/sdk/lib/utils/api/user-library-api";
import { getItemsApi } from "@jellyfin/sdk/lib/utils/api/items-api";
import { getAudioApi } from "@jellyfin/sdk/lib/utils/api/audio-api";
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

    console.log(`device: ${device}, id: ${uuid}`);

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

// export async function testJf() {
//   const auth = await getUserApi(api).authenticateUserByName({
//     authenticateUserByName: {
//       Username: import.meta.env.VITE_JELLYFIN_USER,
//       Pw: import.meta.env.VITE_JELLYFIN_PASSWORD,
//     },
//   });

//   console.log("Auth =>", auth.data);

//   const libraries = await getLibraryApi(api).getMediaFolders();
//   console.log("Libraries =>", libraries.data);

//   const music = libraries.data.Items?.at(0);
//   const musicId = music?.Id;

//   const items = await getUserLibraryApi(api).getItem({ itemId: music?.Id });
//   console.log("Libraries =>", items.data);

//   let children = await getItemsApi(api).getItems({ parentId: musicId });
//   console.log("Libraries =>", children.data);

//   const artistId = children.data.Items?.at(0)?.Id;
//   children = await getItemsApi(api).getItems({ parentId: artistId });
//   console.log("Libraries =>", children.data);

//   const albumId = children.data.Items?.at(2)?.Id;
//   children = await getItemsApi(api).getItems({ parentId: albumId });
//   console.log("Libraries =>", children.data);

//   const songId = children.data.Items?.at(2)?.Id;

//   return getAudioApi(api).getAudioStream({ itemId: songId });
// }
