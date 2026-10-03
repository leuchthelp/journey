<script lang="ts">
  import { Play, Pause } from "@lucide/svelte";

  type Props = {
    action: string;
  };

  let { action }: Props = $props();

  let paused = $state(true);
</script>

<media-play-button class={`${action} playbar-styled-button`}>
  {#if paused}
    <Pause />
  {:else}
    <Play />
  {/if}
</media-play-button>
