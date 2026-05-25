<script lang="ts">
  let { children } = $props();
</script>

<button class="place-self-start" onclick={() => console.log("Clicked")}>
  <a href="/settings/">
    {@render children?.()}
  </a>
</button>
