<script lang="ts">
  import type { ProviderApiError, ProviderDTO } from "#lib/bindings.ts";
  import { strip } from "#lib/components/helpers.ts";
  import { API } from "#lib/proxy.ts";

  const getProvider = async (key?: number): Promise<ProviderDTO | Error> => {
    if (!key) return Error("no key yet");

    return await API.provider
      .get_provider(key)
      .then((response) => {
        return strip(response);
      })
      .catch((err: ProviderApiError) => {
        console.error(err);
        return Error(err);
      });
  };

  type Props = {
    key?: number;
  };

  let { key }: Props = $props();

  let success = $state(false);
  let uname = $state("");
  let psw = $state("");
  let url = $state("");

  let provider = $derived(await getProvider(key));
  $inspect(provider);
</script>

<div class="">
  {#await provider}
    <div>Loading</div>
  {:then provider}
    {#if success}
      <div>Connected</div>
      <div>{key}</div>
      <div>{provider}</div>
      <button onclick={() => console.debug("no impl")}>Remove Connection</button
      >
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
  {/await}
</div>
