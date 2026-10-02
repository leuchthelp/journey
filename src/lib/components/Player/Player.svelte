<script lang="ts">
  import "@videojs/html/audio/player";
  import "@videojs/html/ui/container";
  import * as Playbar from "#lib/components/Playbar/index.ts";
  import type { AudioPlayerElement } from "@videojs/html/audio";
  import { onMount } from "svelte";

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
</script>

<audio-player bind:this={player} class="z-1">
  <media-container>
    {@render children?.()}
  </media-container>
</audio-player>
<Playbar.Root>
  <Playbar.Skip action={"backward"} seconds={"-5"} />
  <Playbar.Button action={"paused"} />
  <Playbar.Skip action={"forward"} seconds={"+15"} />
</Playbar.Root>
