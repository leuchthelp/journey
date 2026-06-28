<script lang="ts">
  import "../app.css";
  import * as Navbar from "$lib/components/Navbar/index.ts";
  import * as Playbar from "$lib/components/Playbar/index.ts";
  import ProviderAccordion from "$lib/components/Settings/Provider/ProviderAccordion.svelte";
  import Settings from "$lib/components/Settings/Settings.svelte";
  import { toAuthComponent } from "$lib/snippets/ToAuthComponent.svelte";
  import type { LayoutProps } from "./$types.d.ts";
  import { providerManager } from "$lib/providers/ProviderManager.ts";
  import { setIndexing } from "$lib/signals/index.svelte.ts";
  import { setMediaSource } from "$lib/signals/mediaSource.svelte.ts";
  import "@videojs/html/audio/player";
  import "@videojs/html/audio/minimal-skin";
  import "@videojs/html/ui/controls";
  import "@videojs/html/ui/play-button";

  function toggleVisible() {
    visible = !visible;
  }

  let { data, children }: LayoutProps = $props();
  let visible = $state(false);

  let displayable: string[] = $state([]);
  function addComponent() {
    displayable.push("JellyfinProvider");
  }

  let signal = $state({ value: { type: "", uuid: "" } });
  setIndexing(signal);

  let src = $state({ url: "" });
  setMediaSource(src);

  $effect(() => {
    providerManager.initProvider(data.post, signal);
  });

  $inspect(src);
</script>

<main
  class="flex mt-5 pl-40 p-2 h-full max-w-full overflow-scroll scrollbar-none overscroll-none"
>
  {@render children()}
</main>

<audio-player class="z-1">
  <media-container>
    <Playbar.Root>
      <Playbar.Skip action={"backward"} seconds={"-5"} />
      <Playbar.Button action={"paused"} />
      <Playbar.Skip action={"forward"} seconds={"+15"} />
    </Playbar.Root>

    <audio src={src.url} autoplay></audio>
  </media-container>
</audio-player>

<div
  class="fixed flex flex-row md:h-full place-self-start *:m-1"
  class:w-full={visible}
>
  <Navbar.Root>
    <Navbar.Button func={toggleVisible}>settings</Navbar.Button>
  </Navbar.Root>
  {#if visible}
    <Settings>
      <ProviderAccordion title={"Providers"}>
        <ProviderAccordion title={"Jellyfin"}>
          <button onclick={() => addComponent()}>Add Jellyfin Provider</button>
          {#each displayable as type}
            {@render toAuthComponent(type)}
          {/each}
          {#each data.post as item}
            {@render toAuthComponent(item.type, item.serverId)}
          {/each}
        </ProviderAccordion>
      </ProviderAccordion>
    </Settings>
  {/if}
</div>
