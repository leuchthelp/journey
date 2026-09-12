<script lang="ts">
  import {
    type IndexerKey,
    type ProviderDTO,
    type ProviderKey,
    type ProviderVariant,
  } from "#lib/bindings.ts";
  import { strip } from "#lib/components/helpers.ts";
  import { API } from "#lib/proxy.ts";
  import { Effect } from "effect";
  import { passwordAuth } from "#lib/effects/auth.ts";

  const removeConnection = async () => {
    if (key == undefined) {
      throw new Error(
        "Key should be set if connection succeeded. Somehow it is not.",
      );
    }

    let tmp = provider?.url;
    if (typeof tmp === "string") url = tmp;

    await API.provider.deregister(key).then((response) => {
      return strip(response);
    });

    key = undefined;
    uname = "";
    psw = "";
  };

  const authenticateProvider = (
    url: string,
    type: ProviderVariant,
    uname: string,
    psw: string,
  ) => {
    key = Effect.runSync(passwordAuth(url, type, uname, psw));
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
      providerId: provider.key?.providerId,
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
        if (incoming.event === "progress") {
          let data = incoming.data;
          if (data.item) msg = data.item;
          console.log(
            `indexer status: ${data.item}, ${data.success}, ${data.alreadyExists}`,
          );
        }
      })
      .then((response) => {
        return strip(response);
      });
  };

  type Props = {
    key?: ProviderKey;
  };
  let { key }: Props = $props();

  let url = $state("");
  let type: ProviderVariant = $state("JellyfinProvider");
  let uname = $state("");
  let psw = $state("");

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
    <form onsubmit={() => authenticateProvider(url, type, uname, psw)}>
      <label for="url">Server Address</label>
      <input type="url" id="url" required bind:value={url} />

      <label for="uname">Username</label>
      <input type="text" id="uname" required bind:value={uname} />

      <label for="psw">Password</label>
      <input type="password" id="psw" required bind:value={psw} />

      <button type="submit">Connect</button>
    </form>
  {/if}
</div>
