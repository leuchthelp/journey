<script lang="ts">
  interface Props {
    styling: string;
    children?: import("svelte").Snippet;
  }

  let { styling, children }: Props = $props();
</script>

<button class={styling} onclick={() => console.log(styling)}>
  {@render children?.()}
</button>
