<script lang="ts">
  import "../app.css";
  import type { LayoutProps } from "./$types";
  import * as Navbar from "#lib/components/Navbar/index.ts";
  import * as Playbar from "#lib/components/Playbar/index.ts";
  import * as ProviderAccordion from "#lib/components/Settings/Provider/index.ts";
  import Settings from "#lib/components/Settings/Settings.svelte";
  import { VariantManager } from "#lib/VariantManager.svelte.ts";

  const toggleVisible = () => {
    visible = !visible;
  };

  let { data, children }: LayoutProps = $props();
  let visible = $state(false);

  let variantManager = $derived(new VariantManager(await data.providerReq));

  let shownVariants = $derived(
    (await data.supportedVariantReq).filter(
      (value) => !variantManager.knownVariants.includes(value),
    ),
  );
</script>

<main
  class="mt-5 flex h-full max-w-full scrollbar-none overflow-scroll overscroll-none p-2 pl-40"
>
  {@render children()}
</main>

<Playbar.Root>
  <Playbar.Skip action={"backward"} seconds={"-5"} />
  <Playbar.Button action={"paused"} />
  <Playbar.Skip action={"forward"} seconds={"+15"} />
</Playbar.Root>

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
        <div>Currently indexing: {variantManager.progress} providers.</div>
        {#each shownVariants as variant}
          {#if variant !== "Unknown"}
            <button onclick={() => variantManager.add(variant)}
              >Add new {variant}</button
            >
          {/if}
        {/each}
        {#each variantManager.all as [variant, proxy] (variant)}
          <ProviderAccordion.Body
            {variant}
            {proxy}
            incProgress={() => variantManager.incProgress()}
            decProgress={() => variantManager.decProgress()}
          />
        {/each}
      </ProviderAccordion.Root>
    </Settings>
  {/if}
</div>
