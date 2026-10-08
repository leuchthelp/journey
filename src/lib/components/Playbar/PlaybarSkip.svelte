<script lang="ts">
  import { SkipForward, SkipBack } from "@lucide/svelte";

  type Props = {
    action: string;
    seconds: string;
  };

  let { action }: Props = $props();
</script>

<button class="{action} playbar-styled-button">
  {#if action === "forward"}
    <SkipForward />
  {:else if action === "backward"}
    <SkipBack />
  {/if}
</button>
