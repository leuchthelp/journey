<script lang="ts">
  import {
    type IndexerKey,
    type ProviderDTO,
    type ProviderKey,
  } from "#lib/bindings.ts";
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
    uname = "";
    psw = "";
  };

  const getProvider = async (
    key?: ProviderKey,
  ): Promise<ProviderDTO | undefined> => {
    if (key == undefined) {
      console.warn("No known provider yet, offering to create new one.");
      return;
    }

    return API.provider.get_provider(key).then((response) => {
      return strip(response);
    });
  };

  function setKey(provider: ProviderDTO | undefined): IndexerKey | undefined {
    if (provider === undefined) {
      return;
    }

    if (provider.key?.providerId === undefined) {
      return;
    }

    let key: IndexerKey = {
      provider_id: provider.key?.providerId,
      variant: provider.type,
    };

    return key;
  }

  const indexerStatus = async (key?: IndexerKey) => {
    if (key == undefined) {
      console.warn("No known provider yet, offering to create new one.");
      return;
    }

    await API.provider
      .indexer_status(key, (incoming) => {
        if (incoming.item) msg = incoming.item;
        console.log(
          `indexer status: ${incoming.item}, ${incoming.success}, ${incoming.already_exists}`,
        );
      })
      .then((response) => {
        return strip(response);
      });
  };

  type Props = {
    key?: ProviderKey;
  };
  let { key }: Props = $props();

  let uname = $state("");
  let psw = $state("");
  let url = $state("");

  let provider = $derived(await getProvider(key));

  let msg: String | undefined = $state(undefined);
  let indexer_key: IndexerKey | undefined = $derived(setKey(provider));
  let indexer_status = $derived(indexerStatus(indexer_key));
  $inspect(provider);
</script>

<div class="">
  {#if provider?.authenticated}
    <div>Connected</div>
    <div>{key}</div>
    <div>{provider.url}</div>
    <button onclick={async () => await removeConnection()}
      >Remove Connection</button
    >
    <div>
      {#await indexer_status}
        <div>In progress: {msg}</div>
      {:then indexer_status}
        <div>Finished Indexing {indexer_status}</div>
      {/await}
    </div>
  {:else}
    <form>
      <label for="url">Server Address</label>
      <input type="url" required bind:value={url} />

      <label for="uname">Username</label>
      <input required bind:value={uname} />

      <label for="psw">Password</label>
      <input type="password" required bind:value={psw} />

      <button onclick={async () => await authenticateProvider(url, uname, psw)}
        >Connect</button
      >
    </form>
  {/if}
</div>
