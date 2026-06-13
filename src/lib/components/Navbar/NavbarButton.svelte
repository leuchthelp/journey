<script lang="ts">
  type Props = {
    func: Function;
    children?: import("svelte").Snippet;
  };

  let { func, children }: Props = $props();
</script>

<button class="place-self-start" onclick={() => func()}>
  {@render children?.()}
</button>
