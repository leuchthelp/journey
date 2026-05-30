<script lang="ts">
  import { providerManager } from "$lib/providers/ProviderManager";
  import { JellyfinProvider } from "$lib/providers/variants";
  import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";
  import { error } from "@sveltejs/kit";

  let success = $state(false);
  let server = $state("");
  let uname = $state("");
  let psw = $state("");
  let authToken = $state("");

  type Props = {
    serverID: string;
  };

  let { serverID }: Props = $props();

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
    if (server === "" || uname === "" || psw === "") {
      return false;
    }

    if (!api) {
      if (authToken !== "") {
        api = provider.createApi(server, authToken);
      } else {
        api = provider.createApi(server);
      }
    }

    let auth = await getUserApi(api).authenticateUserByName({
      authenticateUserByName: {
        Username: uname,
        Pw: psw,
      },
    });

    if (!auth.data.AccessToken) {
      return false;
    }

    authToken = auth.data.AccessToken;

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
    const tmpPsw = localStorage.getItem("jellyfinPsw");
    const tmpUname = localStorage.getItem("jellyfinUname");
    const tmpAuthToken = localStorage.getItem("jellyfinToken");

    if (tmpAuthToken) {
      sendTestRequest().then(() => {
        authToken = tmpAuthToken;
        success = true;
        return;
      });
    }

    if (tmpPsw && tmpUname) {
      sendTestRequest().then(() => {
        psw = tmpPsw;
        uname = tmpUname;
        success = true;
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

  $inspect(
    `success: ${success}, authToken: ${authToken}, uname: ${uname}, psw: ${psw}`,
  );
</script>

<div class="">
  {#if success}
    <div>Connected</div>
    <button onclick={() => removeConnection()}>Remove Connection</button>
  {:else}
    <form>
      <label for="server">Server Address</label>
      <input type="url" required bind:value={server} />

      <label for="uname">Username</label>
      <input required bind:value={uname} />

      <label for="psw">Password</label>
      <input type="password" required bind:value={psw} />

      <button onclick={() => retrieveCredential()}>Connect</button>
    </form>
  {/if}
</div>
