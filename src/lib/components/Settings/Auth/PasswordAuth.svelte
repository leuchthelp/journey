<script lang="ts">
  import {
    type IndexerMsg,
    type ProviderKey,
    type ProviderVariant,
  } from "#lib/bindings.ts";
  import { Effect } from "effect";
  import { passwordAuth, logOutOfProvider } from "#lib/effects/auth.ts";
  import {
    getProvider,
    setIndexerKey,
    indexerStatus,
  } from "#lib/effects/provider.ts";

  const callback = (incoming: IndexerMsg) => {
    if (incoming.event === "Progress") {
      let data = incoming.data;
      if (data.item) msg = data.item;
    }
  };

  type Props = {
    variant: ProviderVariant;
    key?: ProviderKey;
  };
  let { variant, key = $bindable() }: Props = $props();

  let url = $state("");
  let uname = $state("");
  let psw = $state("");
  let msg = $state("");

  let provider = $derived(await Effect.runPromise(getProvider(key)));

  let indexer_key = $derived(Effect.runSync(setIndexerKey(provider)));
  let indexer_status = $derived(
    Effect.runPromise(indexerStatus(callback, indexer_key)),
  );
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
          passwordAuth(url, variant, uname, psw),
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
