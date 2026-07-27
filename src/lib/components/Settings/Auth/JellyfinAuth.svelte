<script lang="ts">
  import { API } from "#lib/proxy.ts";
  import { getIndexing } from "#lib/signals/index.svelte";

  type Props = {
    key?: number;
  };

  let { key }: Props = $props();

  let uname = $state("");
  let psw = $state("");

  let signal = getIndexing();

  let test = await API.provider.get_provider(key);
  $inspect(test);

  let provider = $derived((await API.provider.get_provider(key)).data);

  let success = $derived(provider.authenticated);
  let serverURL = $derived(provider.url);

  $inspect(provider);
</script>

<div class="">
  {#if success}
    <div>Connected</div>
    <div>{key}</div>
    <div>{provider}</div>
    <button onclick={() => console.debug("no impl")}>Remove Connection</button>
  {:else}
    <form>
      <label for="serverURL">Server Address</label>
      <input type="url" required bind:value={serverURL} />

      <label for="uname">Username</label>
      <input required bind:value={uname} />

      <label for="psw">Password</label>
      <input type="password" required bind:value={psw} />

      <button onclick={async () => console.debug("no impl")}>Connect</button>
    </form>
  {/if}
</div>
