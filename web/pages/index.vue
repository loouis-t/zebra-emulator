<template>
  <main>
    <div class="card">
      <h1>Zebra Emulator</h1>
      <p>Enter a virtual printer name to get a live preview URL.</p>
      <div class="row">
        <input
          v-model="name"
          placeholder="my-printer"
          spellcheck="false"
          autofocus
          @keydown.enter="go"
        />
        <button :disabled="!name.trim()" @click="go">Open →</button>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
const name = ref('')
const router = useRouter()
const go = () => {
  const slug = name.value.trim()
  if (!slug) return
  const id = crypto.randomUUID().split('-')[0] // 8 hex chars, enough entropy
  router.push(`/${encodeURIComponent(slug)}-${id}`)
}
</script>

<style scoped>
main {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
}
.card {
  background: #141414;
  border: 1px solid #222;
  border-radius: 14px;
  padding: 40px;
  width: 100%;
  max-width: 420px;
}
h1 { margin: 0 0 8px; font-size: 1.3em; font-weight: 600; }
p { margin: 0 0 24px; font-size: 0.875em; color: #555; line-height: 1.5; }
.row { display: flex; gap: 8px; }
input {
  flex: 1;
  background: #0d0d0d;
  border: 1px solid #2a2a2a;
  border-radius: 7px;
  padding: 10px 13px;
  color: #e0e0e0;
  font-family: monospace;
  font-size: 0.9em;
  outline: none;
  transition: border-color 0.15s;
}
input:focus { border-color: #444; }
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