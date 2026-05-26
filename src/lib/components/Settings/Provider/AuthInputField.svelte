<script lang="ts">
  type Props = {
    title?: string;
  }

  let { title }: Props = $props();
</script>

<form>
    <label for="server">Server Address</label>
    <input>

    <label for="uname">Username</label>
    <input>

    <label for="psw">Password</label>
    <input>
</form>