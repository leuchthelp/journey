<script lang="ts">
  import type { ProviderDTO } from "#lib/bindings.ts";
  import { strip } from "#lib/components/helpers.ts";
  import { API } from "#lib/proxy.ts";

  type Props = {
    key?: number;
  };

  let { key }: Props = $props();

  let success = $state(false);
  let uname = $state("");
  let psw = $state("");
  let url = $state("");

  let provider: ProviderDTO | null = $state(null);

  $effect(() => {
    if (key)
      (async () => {
        provider = await API.provider
          .get_provider(key)
          .then((response) => {
            return strip(response);
          })
          .catch((err) => {
            console.error(err);
            return null;
          });
      })();
  });
</script>

<div class="">
  {#if success}
    <div>Connected</div>
    <div>{key}</div>
    <div>{provider}</div>
    <button onclick={() => console.debug("no impl")}>Remove Connection</button>
  {:else}
    <form>
      <label for="url">Server Address</label>
      <input type="url" required bind:value={url} />

      <label for="uname">Username</label>
      <input required bind:value={uname} />

      <label for="psw">Password</label>
      <input type="password" required bind:value={psw} />

      <button onclick={async () => console.debug("no impl")}>Connect</button>
    </form>
  {/if}
</div>
