<script lang="ts">
  import { getMediaSource } from "$lib/signals/mediaSource.svelte";
  import { store, Player } from "./PlayerControlls.svelte";

  let test = $state(store);
  let src = getMediaSource();

  function play() {
    store.loadSource(src.url);
    if (store.canPlay) store.play();
  }

  $inspect(test, Player);
</script>

<journey-audio-player>
  <button onclick={() => play()}>test</button>
</journey-audio-player>
