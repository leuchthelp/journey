<script lang="ts">
  import { getMediaSource } from "$lib/signals/mediaSource.svelte";
  import { store } from "./PlayerControlls.svelte";

  let test = $state(store);
  let src = getMediaSource();

  function play() {
    store.loadSource(src.url);
    if (store.canPlay) store.play();
  }

  $inspect(test);
</script>

<journey-audio-player>
  <button onclick={() => play()}>test</button>
  <audio
    src="https://music.leuchtapp.com/Audio/f0844cd0869870212759bddecf2c908c/stream?static=true"
  ></audio>
</journey-audio-player>
