<script lang="ts">
  let count = 0;

  function increment() {
    count += 1;
  }
</script>

<div class="container">
  <h1>Forge Svelte App</h1>
  <p>Count: {count}</p>
  <button on:click={increment}>Increment</button>
  <p class="hint">Edit web/App.svelte to get started</p>
</div>

<style>
  .container {
    font-family: system-ui;
    padding: 2rem;
  }
  .hint {
    margin-top: 1rem;
    color: #666;
  }
</style>
