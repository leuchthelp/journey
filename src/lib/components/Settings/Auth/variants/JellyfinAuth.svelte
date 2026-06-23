<script lang="ts">
  import { providerManager } from "$lib/providers/ProviderManager";
  import { JellyfinProvider } from "$lib/providers/variants/JellyfinProvider";
  import { getIndexing } from "$lib/signals/index.svelte";

  async function addConnection() {
    if (!provider.authStatus()) {
      provider.createApi(serverURL);
      await provider.authApiWithPw(uname, psw);
      if (provider.authStatus()) {
        provider
          .indexFiles(signal)
          .finally(() => console.log("auth indexing finished"));
        success = true;
      }
    }
  }

  function removeConnection() {
    provider.removeConnection();
    psw = "";
    success = false;
  }

  type Props = {
    serverId?: string;
  };

  let { serverId }: Props = $props();

  let uname = $state("");
  let psw = $state("");

  let signal = getIndexing();

  let provider = $derived.by(() => {
    if (!serverId) {
      return new JellyfinProvider();
    } else {
      return providerManager.getProviderByServerId(
        serverId,
      ) as JellyfinProvider;
    }
  });

  let success = $derived(provider.authStatus());
  let serverURL = $derived(provider.url);
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

      <button onclick={async () => addConnection()}>Connect</button>
    </form>
  {/if}
</div>
