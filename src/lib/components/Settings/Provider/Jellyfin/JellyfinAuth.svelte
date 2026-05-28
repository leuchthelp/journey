<script lang="ts">
  import { api } from "$lib/providers/JellyfinProvider";
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

    console.log(auth);

    if (!auth.data.AccessToken) {
      return false;
    }

    //success = true
    return true;
  };

  function retrieveCredential() {
    let res = sendTestRequest();

    res.then((value) => {
      console.log(value);
    });
  }
</script>

<div class="">
  {#if success}
    <div>Already in</div>
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
