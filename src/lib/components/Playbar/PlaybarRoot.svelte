<script lang="ts">
  let { children } = $props();
</script>

<media-controls-group class="grid-play-options bottom-0">
  {@render children?.()}
</media-controls-group>
