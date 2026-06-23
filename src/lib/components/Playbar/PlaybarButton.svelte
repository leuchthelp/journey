<script lang="ts">
  type Props = {
    action?: string;
    children?: import("svelte").Snippet;
  };

  let { action, children }: Props = $props();

  $effect(() => {
    if (!action) action = "";
  });
</script>

<button
  class={`${action} playbar-styled-button`}
  onclick={() => console.log(`${action} playbar-styled-button`)}
>
  {@render children?.()}
</button>
