<script lang="ts">
  type Props = {
    styling: string;
  };

  let { styling }: Props = $props();
</script>

<button class={styling} onclick={() => console.log(styling)}>
  <slot />
</button>
