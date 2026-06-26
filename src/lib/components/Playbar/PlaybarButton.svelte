<script lang="ts">
  import { Play, Pause } from "@lucide/svelte";

  function toggleState() {
    state = !state;
  }

  type Props = {
    action?: string;
  };

  let { action }: Props = $props();

  let state = $state(false);

  $effect(() => {
    if (!action) action = "";
  });

  $inspect(state);
</script>

<media-play-button class={`${action} playbar-styled-button`}>
  {#if state}
    <Pause />
  {:else}
    <Play />
  {/if}
</media-play-button>
