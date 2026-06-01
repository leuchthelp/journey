<script lang="ts">
  import { providerManager } from "$lib/providers/ProviderManager";
  import { JellyfinProvider } from "$lib/providers/variants/JellyfinProvider";

  type Props = {
    serverId?: string;
  };

  let { serverId }: Props = $props();

  let uname = $state("");
  let psw = $state("");

  let provider: JellyfinProvider;

  if (!serverId) {
    provider = new JellyfinProvider();
  } else {
    provider = providerManager.getProviderByID(serverId) as JellyfinProvider;
  }

  let success = $state(provider.authStatus());
  let serverURL = $state(provider.url);

  function addConnection() {
    if (!provider.authStatus()) {
      provider.createApi(serverURL);
      provider.authApiWithPw(uname, psw).catch((e) => console.error(e));

      console.log(provider.authStatus());
      if (provider.authStatus()) {
        success = true;
      }
    }
  }

  function removeConnection() {
    provider.removeConnection();
    psw = "";
    success = false;
  }

  $inspect(
    `success: ${success}, uname: ${uname}, psw: ${psw}, url: ${serverURL}, serverId: ${serverId}`,
  );
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

      <button onclick={() => addConnection()}>Connect</button>
    </form>
  {/if}
</div>
