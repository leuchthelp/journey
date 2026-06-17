<script lang="ts">
  let { children } = $props();
</script>

<media-play-button class="media-play-button bottom-0 grid-play-options z-1">
  {@render children?.()}
</media-play-button>
