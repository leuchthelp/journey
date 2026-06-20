<script lang="ts">
  import "../app.css";
  import Navbar from "$lib/components/Navbar/Navbar.svelte";
  import NavbarButton from "$lib/components/Navbar/NavbarButton.svelte";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import PlaybarButton from "$lib/components/Playbar/PlaybarButton.svelte";
  import ProviderAccordion from "$lib/components/Settings/Provider/ProviderAccordion.svelte";
  import Settings from "$lib/components/Settings/Settings.svelte";
  import { toAuthComponent } from "$lib/snippets/ToAuthComponent.svelte";
  import type { LayoutProps } from "./$types";
  import { providerManager } from "$lib/providers/ProviderManager";

  let { data, children }: LayoutProps = $props();
  let visible = $state(false);

  function toggleVisible() {
    visible = !visible;
  }

  let displayables: string[] = $state([]);
  function addComponent() {
    displayables.push("JellyfinProvider");
  }

  // ugly, need to change to remove duplicate code
  $effect(() => {
    providerManager.initProvider(data.post);
  });
</script>

<main class="mt-5 h-full">
  {@render children()}
</main>

<div></div>

<audio-player>
  <media-container>
    <Playbar>
      <PlaybarButton>1</PlaybarButton>
      <PlaybarButton>2</PlaybarButton>
      <PlaybarButton>3</PlaybarButton>
    </Playbar>

    <audio
      src="https://music.leuchtapp.com/Audio/dbc3af6e79261a0429c7c666f7a8031b/stream?static=true"
    ></audio>
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
          {#each displayables as type}
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
