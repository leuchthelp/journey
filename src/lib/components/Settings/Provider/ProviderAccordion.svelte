<script lang="ts">
  import { fade } from "svelte/transition";
  type Props = {
    title: string;
    children?: import("svelte").Snippet;
  };

  let { title, children }: Props = $props();

  let visible = $state(false);

  function toggleVisible() {
    visible = !visible;
  }
</script>

<div class="flex flex-col outline-4 outline-amber-200">
  <button class="place-self-start" onclick={() => toggleVisible()}
    >{title}</button
  >

  {#if visible}
    <div transition:fade={{ duration: 75 }}>
      {@render children?.()}
    </div>
  {/if}
</div>
