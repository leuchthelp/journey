import { Jellyfin } from "@jellyfin/sdk";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { getLibraryApi } from "@jellyfin/sdk/lib/utils/api/library-api";
import { getUserLibraryApi } from "@jellyfin/sdk/lib/utils/api/user-library-api";
import { getItemsApi } from "@jellyfin/sdk/lib/utils/api/items-api";
import { getAudioApi } from "@jellyfin/sdk/lib/utils/api/audio-api";

export async function testJf() {
  const jellyfin = new Jellyfin({
    clientInfo: {
      name: "journey",
      version: "0.1.0",
    },
    deviceInfo: {
      name: "dev",
      id: "dev-pc",
    },
  });

  const api = jellyfin.createApi(
    `https://${import.meta.env.VITE_JELLYFIN_SERVER}/`,
  );
  const auth = await getUserApi(api).authenticateUserByName({
    authenticateUserByName: {
      Username: import.meta.env.VITE_JELLYFIN_USER,
      Pw: import.meta.env.VITE_JELLYFIN_PASSWORD,
    },
  });

  console.log("Auth =>", auth.data);

  const libraries = await getLibraryApi(api).getMediaFolders();
  console.log("Libraries =>", libraries.data);

  const music = libraries.data.Items?.at(0);
  const musicId = music?.Id;

  const items = await getUserLibraryApi(api).getItem({ itemId: music?.Id });
  console.log("Libraries =>", items.data);

  let children = await getItemsApi(api).getItems({ parentId: musicId });
  console.log("Libraries =>", children.data);

  const artistId = children.data.Items?.at(0)?.Id;
  children = await getItemsApi(api).getItems({ parentId: artistId });
  console.log("Libraries =>", children.data);

  const albumId = children.data.Items?.at(2)?.Id;
  children = await getItemsApi(api).getItems({ parentId: albumId });
  console.log("Libraries =>", children.data);

  const songId = children.data.Items?.at(2)?.Id;

  return getAudioApi(api).getAudioStream({ itemId: songId, _static: true });
}
