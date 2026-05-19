import { Jellyfin } from "@jellyfin/sdk";
import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
import { getLibraryApi } from "@jellyfin/sdk/lib/utils/api/library-api";

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


  const api = jellyfin.createApi(`https://${import.meta.env.VITE_JELLYFIN_SERVER}/`);
  const auth = await getUserApi(api).authenticateUserByName({
    authenticateUserByName: {
      Username: import.meta.env.VITE_JELLYFIN_USER,
      Pw: import.meta.env.VITE_JELLYFIN_PASSWORD,
    },
  });

  console.log("Auth =>", auth.data);

  const libraries = await getLibraryApi(api).getMediaFolders();
  console.log("Libraries =>", libraries.data);
}
