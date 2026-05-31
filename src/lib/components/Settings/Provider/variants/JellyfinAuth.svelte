<script lang="ts">
  import { db } from "$lib/db/database";
  import { providerItems, type ProviderItem } from "$lib/db/schema/schema";
  import { providerManager } from "$lib/providers/ProviderManager";
  import { JellyfinProvider } from "$lib/providers/variants";
  import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
  import { error } from "@sveltejs/kit";
  import { eq } from "drizzle-orm";

  let success = $state(false);
  let uname = $state("");
  let psw = $state("");
  let authToken = $state("");

  type Props = {
    id: string;
    url: string;
    type: string;
  };

  let { id: serverID, url: serverURL, type }: Props = $props();

  let provider = providerManager.getProviderByServerID(serverID);

  if (!provider) {
    provider = new JellyfinProvider();
    providerManager.providers.push(provider);
  }

  if (!(provider instanceof JellyfinProvider)) {
    error(404);
  }

  let api = provider.getApi();

  const sendTestRequest = async (): Promise<boolean> => {
    if (serverURL === "" || uname === "" || psw === "") {
      return false;
    }

    if (!api) {
      if (authToken !== "") {
        api = provider.createApi(serverURL, authToken);
      } else {
        api = provider.createApi(serverURL);
      }
    }

    let auth = await getUserApi(api).authenticateUserByName({
      authenticateUserByName: {
        Username: uname,
        Pw: psw,
      },
    });

    console.log(auth);

    if (!auth.data.AccessToken) {
      return false;
    }

    authToken = auth.data.AccessToken;

    if (!serverID) {
      serverID = auth.data.ServerId!;
    }

    console.log(`serverID: ${serverID}, url: ${serverURL}`);

    let addserver: ProviderItem = { id: serverID, url: serverURL, type: type };
    //await db.delete(providerItems)
    await db.insert(providerItems).values(addserver).onConflictDoNothing();

    localStorage.setItem("jellyfinToken", authToken);
    localStorage.setItem("jellyfinPsw", psw);
    localStorage.setItem("jellyfinUname", uname);
    return true;
  };

  function retrieveCredential() {
    let res = sendTestRequest();

    res.then((value) => {
      success = value;
    });
  }

  tryExistingCreds();

  function tryExistingCreds() {
    success = false;

    const tmpPsw = localStorage.getItem("jellyfinPsw");
    const tmpUname = localStorage.getItem("jellyfinUname");
    const tmpAuthToken = localStorage.getItem("jellyfinToken");

    db.select()
      .from(providerItems)
      .where(eq(providerItems.id, serverID))
      .then((serverItem) => {
        serverURL = serverItem.at(0)?.url!;
      });

    console.log("first");
    if (tmpAuthToken) {
      sendTestRequest().then((check) => {
        if (check) {
          authToken = tmpAuthToken;
          success = true;
        }
        return;
      });
    }

    console.log("second");

    if (tmpPsw && tmpUname) {
      sendTestRequest().then((check) => {
        if (check) {
          psw = tmpPsw;
          uname = tmpUname;
          success = true;
        }
        return;
      });
    }
  }

  function removeConnection() {
    localStorage.removeItem("jellyfinPsw");
    localStorage.removeItem("jellyfinUname");
    localStorage.removeItem("jellyfinToken");
    authToken = "";
    psw = "";
    success = false;
  }

  // $inspect(
  //   `success: ${success}, authToken: ${authToken}, uname: ${uname}, psw: ${psw}`,
  // );
</script>

<div class="">
  {#if success}
    <div>Connected</div>
    <button onclick={() => removeConnection()}>Remove Connection</button>
  {:else}
    <form>
      <label for="serverURL">Server Address</label>
      <input type="url" required bind:value={serverURL} />

      <label for="uname">Username</label>
      <input required bind:value={uname} />

      <label for="psw">Password</label>
      <input type="password" required bind:value={psw} />

      <button onclick={() => retrieveCredential()}>Connect</button>
    </form>
  {/if}
</div>
