<script lang="ts">
  import { getIndexing } from "#lib/signals/index.svelte";

  type Props = {
    userId?: string;
  };

  let { userId }: Props = $props();

  let uname = $state("");
  let psw = $state("");

  let signal = getIndexing();

  let provider = $derived.by(() => {
    console.log(signal);
  });

  let success = $derived(false);
  let serverURL = $derived("");
</script>

<div class="">
  {#if success}
    <div>Connected</div>
    <div>{userId}</div>
    <div>{provider}</div>
    <button onclick={() => console.debug("no impl")}>Remove Connection</button>
  {:else}
    <form>
      <label for="serverURL">Server Address</label>
      <input type="url" required bind:value={serverURL} />

      <label for="uname">Username</label>
      <input required bind:value={uname} />

      <label for="psw">Password</label>
      <input type="password" required bind:value={psw} />

      <button onclick={async () => console.debug("no impl")}>Connect</button>
    </form>
  {/if}
</div>
