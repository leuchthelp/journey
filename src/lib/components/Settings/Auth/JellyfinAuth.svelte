<script lang="ts">
  import { type ProviderDTO } from "#lib/bindings.ts";
  import { strip } from "#lib/components/helpers.ts";
  import { API } from "#lib/proxy.ts";

  const removeConnection = async () => {
    if (key == undefined) {
      throw new Error(
        "Key should be set if connection succeeded. Somehow it is not.",
      );
    }

    let tmp = (await provider)?.url;
    if (typeof tmp === "string") url = tmp;

    await API.provider.deregister(key).then((response) => {
      return strip(response);
    });

    key = undefined;
    uname = "";
    psw = "";
  };

  const authenticateProvider = async (
    url: string,
    uname: string,
    psw: string,
  ) => {
    let response = await API.provider
      .password_auth(url, "JellyfinProvider", uname, psw)
      .then((response) => {
        return strip(response);
      });

    key = response;
  };

  const getProvider = async (
    key?: [string, string],
  ): Promise<ProviderDTO | undefined> => {
    if (key == undefined) {
      console.warn("No known provider yet, offering to create new one.");
      return;
    }

    return API.provider.get_provider(key).then((response) => {
      return strip(response);
    });
  };

  type Props = {
    key?: [string, string];
  };

  let { key }: Props = $props();

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
    {#if provider?.authenticated}
      <div>Connected</div>
      <div>{key}</div>
      <div>{provider.url}</div>
      <button onclick={async () => await removeConnection()}
        >Remove Connection</button
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
