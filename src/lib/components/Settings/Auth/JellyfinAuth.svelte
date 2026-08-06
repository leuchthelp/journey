<script lang="ts">
  import type { ProviderApiError, ProviderDTO } from "#lib/bindings.ts";
  import { strip } from "#lib/components/helpers.ts";
  import { API } from "#lib/proxy.ts";

  const authenticateProvider = async (
    url: string,
    uname: string,
    psw: string,
  ) => {
    let response = await API.provider
      .password_auth(url, "JellyfinProvider", uname, psw)
      .then((response) => {
        return strip(response);
      })
      .catch((err: ProviderApiError) => {
        throw Error(err);
      });

    key = response;
    success = true;
  };

  const getProvider = async (
    key?: [string, string],
  ): Promise<ProviderDTO | undefined> => {
    if (!key) {
      console.warn("No known provider yet, offering to create new one.");
      return;
    }

    return API.provider
      .get_provider(key)
      .then((response) => {
        return strip(response);
      })
      .catch((err: ProviderApiError) => {
        throw Error(err);
      });
  };

  type Props = {
    key?: [string, string];
  };

  let { key }: Props = $props();

  let success = $state(false);
  let uname = $state("");
  let psw = $state("");
  let url = $state("");

  let provider = $derived(getProvider(key));
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

        <button
          onclick={async () => await authenticateProvider(url, uname, psw)}
          >Connect</button
        >
      </form>
    {/if}
  {/await}
</div>
