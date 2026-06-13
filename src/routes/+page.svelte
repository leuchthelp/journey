<script lang="ts">
  import type { PageProps } from "./$types";
  import { itemCache, homeCache } from "$lib/components/MediaItems/ItemCache";
  import { toMediaItemComponent } from "$lib/snippets/ToMediaItemComponent.svelte";
  import "@videojs/html/audio/player";
  import "@videojs/html/audio/minimal-skin";

  let { data }: PageProps = $props();

  $inspect(data);
</script>

<audio-player>
  <media-container>
    <audio src="https://music.leuchtapp.com/Audio/07d497d9910f1016ae24263e8bb2262e/stream?static=true"></audio>
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

<div class="flex">
  {#each data.post as item}
    {#if itemCache.set(item.uuid!, item) && homeCache.set(item.uuid!, item)}
      {@render toMediaItemComponent(item)}
    {/if}
  {/each}
</div>
