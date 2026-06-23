<script lang="ts">
  let { children } = $props();
</script>

<media-controls-group class="bottom-0 grid-play-options">
  {@render children?.()}
</media-controls-group>
