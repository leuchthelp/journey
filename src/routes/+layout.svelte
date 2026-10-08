<script lang="ts">
  import "../app.css";
  import type { LayoutProps } from "./$types";
  import * as Navbar from "#lib/components/Navbar/index.ts";
  import * as Playbar from "#lib/components/Playbar/index.ts";
  import * as ProviderAccordion from "#lib/components/Settings/Provider/index.ts";
  import Settings from "#lib/components/Settings/Settings.svelte";
  import { ProviderTypeManager } from "#lib/ProviderTypeManager.svelte.ts";

  const toggleVisible = () => {
    visible = !visible;
  };

  let { data, children }: LayoutProps = $props();
  let visible = $state(false);

  let providerTypeManager = $derived(
    new ProviderTypeManager(await data.providerReq),
  );

  let shownTypes = $derived(
    (await data.supportedTypeReq).filter(
      (value) => !providerTypeManager.knownTypes.includes(value),
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
  <Playbar.Button />
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
        <div>Currently indexing: {providerTypeManager.progress} providers.</div>
        {#each shownTypes as type}
          {#if type !== "Unknown"}
            <button onclick={() => providerTypeManager.add(type)}
              >Add new {type}</button
            >
          {/if}
        {/each}
        {#each providerTypeManager.all as [type, proxy] (type)}
          <ProviderAccordion.Body
            {type}
            {proxy}
            incProgress={() => providerTypeManager.incProgress()}
            decProgress={() => providerTypeManager.decProgress()}
          />
        {/each}
      </ProviderAccordion.Root>
    </Settings>
  {/if}
</div>
