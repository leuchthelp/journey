<script lang="ts">
  interface Props {
    children?: import("svelte").Snippet;
  }

  let { children }: Props = $props();
</script>

<button class="playbar-styled-button" onclick={() => console.log("Clicked")}>
  {@render children?.()}
</button>
