<script lang="ts">
  import "../app.css";
  import * as Navbar from "#lib/components/Navbar/index.ts";
  import * as Playbar from "#lib/components/Playbar/index.ts";
  import ProviderAccordion from "#lib/components/Settings/Provider/ProviderAccordion.svelte";
  import Settings from "#lib/components/Settings/Settings.svelte";
  // import Player from "#lib/components/Player/Player.svelte";
  import { SvelteMap } from "svelte/reactivity";
  import type { ProviderKey, ProviderVariant } from "#lib/bindings.ts";
  import { Effect } from "effect";
  import {
    getProviders,
    getSupportedProviderVariants,
  } from "#lib/effects/provider.ts";
  import ProviderAccordionBody from "#lib/components/Settings/Provider/ProviderAccordionBody.svelte";

  const toggleVisible = () => {
    visible = !visible;
  };

  const addVariant = (type: ProviderVariant, key?: ProviderKey) => {
    let wrapped = $state(key);

    knownVariants.getOrInsert(type, [wrapped]).push(wrapped);
  };

  let { children } = $props();
  let visible = $state(false);

  let supportedVariants = $state(
    await Effect.runPromise(getSupportedProviderVariants()),
  );

  let providers = $state(await Effect.runPromise(getProviders()));
  let knownVariants = $derived.by(() => {
    let map = new SvelteMap<ProviderVariant, (ProviderKey | undefined)[]>();

    for (var provider of providers) {
      let wrapped = $state(provider.key);

      map.getOrInsert(provider.type, [wrapped]).push(wrapped);
    }

    return map;
  });

  $inspect(supportedVariants);
  $inspect(providers);
  $inspect(knownVariants);
</script>

<main
  class="mt-5 flex h-full max-w-full scrollbar-none overflow-scroll overscroll-none p-2 pl-40"
>
  {@render children()}
</main>

<!-- <Player /> -->
<audio-player class="z-1">
  <media-container>
    <Playbar.Root>
      <Playbar.Skip action={"backward"} seconds={"-5"} />
      <Playbar.Button action={"paused"} />
      <Playbar.Skip action={"forward"} seconds={"+15"} />
    </Playbar.Root>

    <audio src={""}></audio>
  </media-container>
</audio-player>

<div
  class="fixed flex flex-row place-self-start *:m-1 md:h-full"
  class:w-full={visible}
>
  <Navbar.Root>
    <Navbar.Button func={toggleVisible}>settings</Navbar.Button>
  </Navbar.Root>
  {#if visible}
    <Settings>
      <ProviderAccordion title={"Providers"}>
        {#each supportedVariants as variant}
          {#if variant !== "Unknown"}
            <button onclick={() => addVariant(variant)}
              >Add new {variant}</button
            >
          {/if}
        {/each}
        {#each knownVariants as [variant, knownKeys] (variant)}
          {#if variant !== "Unknown"}
            <ProviderAccordionBody {variant} bind:knownKeys {addVariant}
            ></ProviderAccordionBody>
          {/if}
        {/each}
      </ProviderAccordion>
    </Settings>
  {/if}
</div>
