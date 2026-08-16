<script lang="ts">
  import "../app.css";
  import * as Navbar from "#lib/components/Navbar/index.ts";
  import * as Playbar from "#lib/components/Playbar/index.ts";
  import ProviderAccordion from "#lib/components/Settings/Provider/ProviderAccordion.svelte";
  import Settings from "#lib/components/Settings/Settings.svelte";
  import { toAuthComponent } from "#lib/snippets/ToAuthComponent.svelte";
  import "@videojs/html/audio/player";
  import "@videojs/html/audio/minimal-skin";
  import "@videojs/html/ui/controls";
  import "@videojs/html/ui/play-button";
  import Player from "#lib/components/Player/Player.svelte";
  import { API } from "#lib/proxy.ts";
  import type { ProviderDTO, ProviderVariant } from "#lib/bindings.ts";

  function toggleVisible() {
    visible = !visible;
  }

  let { children } = $props();
  let visible = $state(false);

  let displayable: ProviderVariant[] = $state([]);
  function addComponent() {
    displayable.push("JellyfinProvider");
  }

  let data: ProviderDTO[] = await API.provider
    .get_providers()
    .then((result) => {
      if (result.status == "ok") {
        return result.data;
      } else {
        return [];
      }
    })
    .catch((err) => {
      console.log(err);
      return [];
    });

  let providers = $state(data);

  $inspect(providers);
</script>

<main
  class="mt-5 flex h-full max-w-full scrollbar-none overflow-scroll overscroll-none p-2 pl-40"
>
  {@render children()}
</main>

<Player />
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
        <ProviderAccordion title={"Jellyfin"}>
          <button onclick={() => addComponent()}>Add Jellyfin Provider</button>
          {#each displayable as type}
            {@render toAuthComponent(type)}
          {/each}
          {#each providers as provider}
            {#if provider.key}
              {@render toAuthComponent(provider.type, provider.key)}
            {:else}
              error
            {/if}
          {/each}
        </ProviderAccordion>
      </ProviderAccordion>
    </Settings>
  {/if}
</div>
