<script lang="ts">
  import "@videojs/html/audio/player";
  import "@videojs/html/ui/container";
  import * as Playbar from "#lib/components/Playbar/index.ts";
  import { Play, Pause } from "@lucide/svelte";
  import type { AudioPlayerElement } from "@videojs/html/audio";
  import { onMount } from "svelte";
  import { SkipForward, SkipBack } from "@lucide/svelte";

  type Props = {
    children?: import("svelte").Snippet;
  };

  let { children }: Props = $props();

  let player: AudioPlayerElement;
  let paused = $state(true);

  onMount(() => {
    const sync = () => (paused = player.store.paused);
    sync();
    return player.store.subscribe(sync);
  });

  $inspect(paused);
</script>

<audio-player bind:this={player} class="z-1">
  <media-container>
    <audio
      src="https://music.leuchtapp.com/Audio/f0844cd0869870212759bddecf2c908c/stream?static=true"
    ></audio>
    <Playbar.Root>
      <media-seek-button seconds="-5" class="backward playbar-styled-button">
        <SkipBack />
      </media-seek-button>
      <media-play-button class="playbar-styled-button">
        <span class="paused">Play</span>
        <!-- {#if paused}
          <Play />
        {:else}
          <Pause />
        {/if} -->
      </media-play-button>
      <media-seek-button seconds="+5" class="forward playbar-styled-button">
        <SkipForward />
      </media-seek-button>
    </Playbar.Root>
  </media-container>
</audio-player>
