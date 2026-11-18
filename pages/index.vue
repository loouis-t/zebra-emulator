<template>
  <main>
    <div class="surface card">
      <h1>Zebra Emulator</h1>
      <p>Enter a virtual printer name to get a live preview URL.</p>
      <form class="row" @submit.prevent="go">
        <input v-model="name" placeholder="my-printer" spellcheck="false" autofocus />
        <button type="submit" :disabled="!name.trim()">Open →</button>
      </form>
    </div>
  </main>
</template>

<script setup lang="ts">
const name = ref('')
const go = () => navigateTo(`/${encodeURIComponent(name.value.trim())}-${crypto.randomUUID().split('-')[0]}`)
</script>

<style scoped>
main {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
}
.card {
  border-radius: 14px;
  padding: 40px;
  width: 100%;
  max-width: 420px;
}
h1 { margin: 0 0 8px; font-size: 1.3em; font-weight: 600; }
p { margin: 0 0 24px; font-size: 0.875em; color: #555; line-height: 1.5; }
.row { display: flex; gap: 8px; }
input { flex: 1; padding: 10px 13px; font-family: monospace; font-size: 0.9em; }
button {
  background: #e0e0e0;
  color: #000;
  border: none;
  border-radius: 7px;
  padding: 10px 16px;
  font-weight: 600;
  font-size: 0.875em;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s;
}
button:hover:not(:disabled) { background: #fff; }
button:disabled { opacity: 0.25; cursor: default; }
</style>