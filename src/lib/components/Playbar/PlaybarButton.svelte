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

<media-play-button class={`${action} playbar-styled-button`}>
  {@render children?.()}
</media-play-button>
