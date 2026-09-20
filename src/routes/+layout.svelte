<script lang="ts">
  import "../app.css";
  import * as Navbar from "#lib/components/Navbar/index.ts";
  import * as Playbar from "#lib/components/Playbar/index.ts";
  import ProviderAccordion from "#lib/components/Settings/Provider/ProviderAccordion.svelte";
  import Settings from "#lib/components/Settings/Settings.svelte";
  // import Player from "#lib/components/Player/Player.svelte";
  import type { ProviderVariant } from "#lib/bindings.ts";
  import { Effect } from "effect";
  import {
    getProviders,
    getSupportedProviderVariants,
  } from "#lib/effects/provider.ts";
  import ProviderAccordionBody from "#lib/components/Settings/Provider/ProviderAccordionBody.svelte";
  import { VariantManager } from "#lib/VariantManager.svelte.ts";

  const toggleVisible = () => {
    visible = !visible;
  };

  const filterSupported = (
    supported: ProviderVariant[],
    already_known: ProviderVariant[],
  ) => {
    return supported.filter((value) => !already_known.includes(value));
  };

  let { children } = $props();
  let visible = $state(false);

  let providers = $state(await Effect.runPromise(getProviders()));
  let variantManager = new VariantManager(providers);

  let supportedVariants = $state(
    await Effect.runPromise(getSupportedProviderVariants()),
  );
  let shownVariants = $derived(
    filterSupported(supportedVariants, variantManager.knownVariants),
  );

  $inspect(shownVariants)
  $inspect(variantManager)
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
      <p>Stuff is {JSON.stringify(variantManager.all)}</p>
      <ProviderAccordion title={"Providers"}>
        {#each shownVariants as variant}
          {#if variant !== "Unknown"}
            <button onclick={() => variantManager.add(variant)}
              >Add new {variant}</button
            >
          {/if}
        {/each}
        {#each variantManager.all as proxy (proxy.name)}
          <ProviderAccordionBody {proxy}></ProviderAccordionBody>
        {/each}
      </ProviderAccordion>
    </Settings>
  {/if}
</div>
