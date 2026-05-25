<script lang="ts">
  let { children } = $props();
</script>

<button class="playbar-styled-button" onclick={() => console.log("Clicked")}>
  {@render children?.()}
</button>
