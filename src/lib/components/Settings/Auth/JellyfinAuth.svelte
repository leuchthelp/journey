<script lang="ts">
  import {
    type IndexerKey,
    type IndexerMsg,
    type ProviderDTO,
    type ProviderKey,
    type ProviderVariant,
  } from "#lib/bindings.ts";
  import { Effect } from "effect";
  import { passwordAuth, logOutOfProvider } from "#lib/effects/auth.ts";
  import { getProvider, indexerStatus } from "#lib/effects/provider.ts";

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

  const callback = (incoming: IndexerMsg) => {
    if (incoming.event === "Progress") {
      let data = incoming.data;
      if (data.item) msg = data.item;
      console.log(
        `indexer status: ${data.item}, ${data.success}, ${data.alreadyExists}`,
      );
    }
  };

  type Props = {
    type: ProviderVariant;
    key?: ProviderKey;
  };
  let { type, key }: Props = $props();

  let url: string = $state("");
  let uname: string = $state("");
  let psw: string = $state("");

  let provider: ProviderDTO | undefined = $derived(
    await Effect.runPromise(getProvider(key)),
  );

  let msg: String = $state("");
  let indexer_key: IndexerKey | undefined = $derived(setKey(provider));
  let indexer_status: Promise<boolean> = $derived(
    Effect.runPromise(indexerStatus(callback, indexer_key)),
  );
  $inspect(provider);
</script>

<div class="">
  {#if provider?.authenticated}
    <div>Connected</div>
    <div>{key}</div>
    <div>{provider.url}</div>
    <button
      onclick={async () =>
        (key = await Effect.runPromise(logOutOfProvider(key)))}
      >Remove Connection</button
    >
    <div>
      {#await indexer_status}
        <div>In progress: {msg}</div>
      {:then indexer_status}
        {#if indexer_status}
          <div>Finished Indexing {indexer_status}</div>
        {:else}
          <div>Failed Indexing {indexer_status}</div>
        {/if}
      {/await}
    </div>
  {:else}
    <form
      onsubmit={async () => {
        const [newKey, newUname, newPsw] = await Effect.runPromise(
          passwordAuth(url, type, uname, psw),
        );

        key = newKey;
        uname = newUname;
        psw = newPsw;
      }}
    >
      <label for="url">Server Address</label>
      <input
        type="url"
        id="url"
        required
        bind:value={url}
        placeholder="https://example.com"
        pattern="https?://.*"
      />

      <label for="uname">Username</label>
      <input type="text" id="uname" required bind:value={uname} />

      <label for="psw">Password</label>
      <input
        type="password"
        id="psw"
        required
        bind:value={psw}
        autocomplete="current-password"
      />

      <button type="submit">Connect</button>
    </form>
  {/if}
</div>
