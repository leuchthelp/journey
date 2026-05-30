<script lang="ts">
  import { api } from "$lib/providers/variants/JellyfinProvider";
  import { getUserApi } from "@jellyfin/sdk/lib/utils/api/user-api";

  let success = $state(false);
  let server = $state("");
  let uname = $state("");
  let psw = $state("");

  const sendTestRequest = async (): Promise<boolean> => {
    if (server === "" || uname === "" || psw === "") {
      return false;
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

    localStorage.setItem("jellyfinToken", auth.data.AccessToken);
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

    if (tmpPsw && tmpUname) {
      sendTestRequest().then(() => {
        psw = tmpPsw;
        uname = tmpUname;
        success = true;
      });
    }
  }

  function removeConnection() {
    localStorage.removeItem("jellyfinPsw");
    localStorage.removeItem("jellyfinUname");
    localStorage.removeItem("jellyfinToken");
    psw = ""
    success = false;
  }
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
