<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import ConversationInterface from './ConversationInterface.svelte';

  let name = 'World';
  let response = '';

  onMount(async () => {
    // Example of calling a Rust command
    // response = await invoke('greet', { name: 'Svelte' });
  });
</script>

<main>
  <h1>Welcome to Oxide Pilot!</h1>
  <p>This is the main application layout.</p>

  <ConversationInterface />
</main>

<style>
  main {
    font-family: sans-serif;
    text-align: center;
    padding: 20px;
  }
  input {
    padding: 8px;
    margin-right: 10px;
  }
  button {
    padding: 8px 15px;
    cursor: pointer;
  }
</style>
