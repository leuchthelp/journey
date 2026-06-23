<script lang="ts">
  import "../app.css";
  import Navbar from "$lib/components/Navbar/Navbar.svelte";
  import NavbarButton from "$lib/components/Navbar/NavbarButton.svelte";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import PlaybarButton from "$lib/components/Playbar/PlaybarButton.svelte";
  import PlaybarSkip from "$lib/components/Playbar/PlaybarSkip.svelte";
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

  $inspect(src)
</script>

<main class="mt-5 h-full">
  {@render children()}
</main>

<div></div>

<audio-player class="z-1">
  <media-container>
    <Playbar>
      <PlaybarSkip action={"backward"} seconds={"-5"}>1</PlaybarSkip>
      <PlaybarButton action={"paused"}>2</PlaybarButton>
      <PlaybarSkip action={"forward"} seconds={"+15"}>3</PlaybarSkip>
    </Playbar>

    <audio src={src.url} autoplay></audio>
  </media-container>
</audio-player>

<div class="fixed flex flex-row h-full place-self-start" class:w-full={visible}>
  <Navbar>
    <div class="h-full"></div>
    <NavbarButton func={toggleVisible}>settings</NavbarButton>
  </Navbar>
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
