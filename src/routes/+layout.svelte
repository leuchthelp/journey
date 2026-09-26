<script lang="ts">
  import "../app.css";
  import type { LayoutProps } from "./$types";
  import * as Navbar from "#lib/components/Navbar/index.ts";
  import * as Playbar from "#lib/components/Playbar/index.ts";
  import * as ProviderAccordion from "#lib/components/Settings/Provider/index.ts";
  import Settings from "#lib/components/Settings/Settings.svelte";
  // import Player from "#lib/components/Player/Player.svelte";
  import { VariantManager } from "#lib/VariantManager.svelte.ts";
  import { Effect } from "effect";
  import { getIndexerProgress } from "#lib/effects/provider.ts";
  import type { ProgressTrackerMsg } from "#lib/bindings.ts";

  const toggleVisible = () => {
    visible = !visible;
  };

  const progressCallback = (response: ProgressTrackerMsg) => {
    switch (response.event) {
      case "Progress":
        progressMessage = `Currently indexing: ${response.data} providers.`;
        break;
    }
  };

  let { data, children }: LayoutProps = $props();
  let visible = $state(false);

  let variantManager = $derived(new VariantManager(await data.providerReq));

  let shownVariants = $derived(
    (await data.supportedVariantReq).filter(
      (value) => !variantManager.knownVariants.includes(value),
    ),
  );

  let progressMessage = $state("");
  let indexerProgress = $state(
    Effect.runPromise(getIndexerProgress(progressCallback)),
  );

  $inspect(indexerProgress);
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
      <ProviderAccordion.Root title={"Providers"}>
        {#await indexerProgress then progress}
          {#if progress}
            <div>{progressMessage}</div>
          {/if}
        {/await}

        {#each shownVariants as variant}
          {#if variant !== "Unknown"}
            <button onclick={() => variantManager.add(variant)}
              >Add new {variant}</button
            >
          {/if}
        {/each}
        {#each variantManager.all as [variant, proxy] (variant)}
          <ProviderAccordion.Body {variant} {proxy} />
        {/each}
      </ProviderAccordion.Root>
    </Settings>
  {/if}
</div>
