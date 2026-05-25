<script lang="ts">
  type Props = {
    children?: import("svelte").Snippet;
  };

  let { children }: Props = $props();
</script>

<div class="flex h-screen w-full place-self-start">
  <div
    class="flex flex-col rounded-2xl m-2 p-3 backdrop-blur-sm backdrop-saturate-125 shadow-xl w-full"
  >
    {@render children?.()}
  </div>
</div>
