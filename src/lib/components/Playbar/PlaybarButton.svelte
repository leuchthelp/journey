<script lang="ts">
  import { play } from "#lib/effects/player.ts";
  import { Play, Pause } from "@lucide/svelte";
  import { Effect } from "effect";

  let state = $state(true);
</script>

<button
  class="playbar-styled-button"
  onclick={async () => await Effect.runPromise(play(false))}
>
  {#if state}
    <Pause />
  {:else}
    <Play />
  {/if}
</button>
