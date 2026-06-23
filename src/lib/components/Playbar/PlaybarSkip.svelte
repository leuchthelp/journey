<script lang="ts">
  type Props = {
    action: string;
    seconds: string;
    children?: import("svelte").Snippet;
  };

  let { action, seconds, children }: Props = $props();
</script>

<media-seek-button {seconds} class={`${action} playbar-styled-button`}>
  {@render children?.()}
</media-seek-button>
