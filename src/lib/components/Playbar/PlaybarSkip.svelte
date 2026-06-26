<script lang="ts">
  import { SkipForward, SkipBack } from "@lucide/svelte";

  type Props = {
    action: string;
    seconds: string;
  };

  let { action, seconds }: Props = $props();
</script>

<media-seek-button {seconds} class={`${action} playbar-styled-button`}>
  {#if action === "forward"}
    <SkipForward />
  {:else if action === "backward"}
    <SkipBack />
  {/if}
</media-seek-button>
