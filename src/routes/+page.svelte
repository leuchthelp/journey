<script lang="ts">
  import type { PageData, PageProps, Snapshot } from "./$types";
  import { itemCache, homeCache } from "$lib/components/MediaItems/ItemCache";
  import { toSvelteComponent } from "$lib/snippets/ToSvelteComponent.svelte";
  import "@videojs/html/audio/player";
  import "@videojs/html/audio/minimal-skin";

  let { data }: PageProps = $props();

  export const snapshot: Snapshot<PageData> = {
    capture: () => data,
    restore: (value) => (data = value),
  };
</script>

<audio-player>
  <media-container>
    <audio></audio>
    <media-play-button class="media-play-button">
      <span class="paused">Play</span>
    </media-play-button>
    <media-volume-slider class="media-volume-slider">
      <media-slider-track class="media-slider-track">
        <media-slider-fill class="media-slider-fill"></media-slider-fill>
      </media-slider-track>
      <media-slider-thumb class="media-slider-thumb"></media-slider-thumb>
      <media-slider-value type="pointer" class="media-slider-value"
      ></media-slider-value>
    </media-volume-slider>
  </media-container>
</audio-player>

{#each data.post as item}
  {#if itemCache.set(item.hash, item) && homeCache.set(item.hash, item)}
    {@render toSvelteComponent(item)}
  {/if}
{/each}
