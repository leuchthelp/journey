<script lang="ts">
  import type { PageData, PageProps, Snapshot } from "./$types";
  import { itemCache, homeCache } from "$lib/components/MediaItems/ItemCache";
  import { toSvelteComponent } from "$lib/snippets/ToSvelteComponent.svelte";
  import type { MediaPlayerElement } from "vidstack/elements";
  import "vidstack/bundle";

  let { data }: PageProps = $props();

  let player: MediaPlayerElement;

  export const snapshot: Snapshot<PageData> = {
    capture: () => data,
    restore: (value) => (data = value),
  };
</script>

<media-player bind:this={player} streamType="on-demand">
  <media-provider>
    <source
      src={data.url.config.url}
      type={`${data.url.headers["content-type"]}`}
    />
  </media-provider>
</media-player>

{#each data.post as item}
  {#if itemCache.set(item.hash, item) && homeCache.set(item.hash, item)}
    {@render toSvelteComponent(item)}
  {/if}
{/each}

<button
  class="playbar-styled-button"
  onclick={() => {
    player.play();
  }}
>
  test
</button>
