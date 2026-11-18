<template>
  <main>
    <header>
      <NuxtLink to="/" class="back">← Printers</NuxtLink>
      <h1>{{ printer }}</h1>
    </header>

    <section class="endpoint">
      <span class="label">Printer Address</span>
      <div class="copy-row">
        <code class="surface">{{ endpoint }}</code>
        <button class="copy-btn" @click="copy">{{ copied ? '✓ Copied' : 'Copy' }}</button>
      </div>
    </section>

    <section class="config">
      <span class="label">Label size (inches)</span>
      <div class="config-row">
        <input v-model.number="size.width" type="number" step="0.25" min="0.5" max="12" />
        <span class="sep">×</span>
        <input v-model.number="size.height" type="number" step="0.25" min="0.5" max="12" />
      </div>
    </section>

    <section class="history">
      <div v-if="!labels.length" class="surface preview empty">Waiting for label…</div>
      <div v-for="(label, i) in labels" :key="label.id" class="surface preview" :class="{ past: i > 0 }">
        <img :src="label.src" alt="Rendered label" />
      </div>
    </section>

    <span class="status">{{ status }}</span>
  </main>
</template>

<script setup lang="ts">
const printer = useRoute().params.printer as string
const endpoint = `${location.host}/${printer}`
const sizeKey = `label-size:${printer}`

const labels = shallowRef<{ id: number; src: string }[]>([])
const status = ref('Connecting…')
const copied = ref(false)
const size = ref(JSON.parse(localStorage.getItem(sizeKey)!) ?? { width: 2.25, height: 1.25 })
const es = new EventSource(`/${printer}/stream`)
let nextId = 0

watch(size, value => localStorage.setItem(sizeKey, JSON.stringify(value)), { deep: true })

es.onopen = () => (status.value = '● Live')
es.onerror = () => (status.value = 'Disconnected — retrying…')
es.onmessage = async (e) => {
  try {
    const images = await renderZpl(e.data, size.value)
    labels.value = [...images.reverse().map(src => ({ id: nextId++, src })), ...labels.value].slice(0, 20)
    status.value = `Updated ${new Date().toLocaleTimeString()}`
  } catch (err) {
    status.value = `Invalid ZPL: ${(err as Error).message}`
  }
}
onUnmounted(() => es.close())

function copy() {
  navigator.clipboard.writeText(endpoint)
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}
</script>

<style scoped>
main { max-width: 700px; margin: 48px auto; padding: 0 24px; }

header { display: flex; align-items: baseline; gap: 14px; margin-bottom: 28px; }
.back { font-size: 0.85em; color: #444; text-decoration: none; transition: color 0.15s; }
.back:hover { color: #aaa; }
h1 { margin: 0; font-size: 1.25em; font-family: monospace; font-weight: 600; }

.endpoint, .config { margin-bottom: 20px; }
.label {
  display: block;
  font-size: 0.7em;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: #444;
  margin-bottom: 7px;
}
.copy-row { display: flex; gap: 8px; align-items: stretch; }
code {
  flex: 1;
  border-radius: 7px;
  padding: 10px 13px;
  font-size: 0.85em;
  word-break: break-all;
  color: #bbb;
}
.copy-btn {
  background: #1a1a1a;
  color: #999;
  border: 1px solid #2a2a2a;
  border-radius: 7px;
  padding: 8px 14px;
  font-size: 0.8em;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s, color 0.15s;
  min-width: 72px;
}
.copy-btn:hover { background: #222; color: #ddd; }

.config-row { display: flex; align-items: center; gap: 8px; }
.config-row input { width: 72px; padding: 9px 11px; font-size: 0.875em; text-align: center; }
.sep { color: #444; font-size: 0.9em; }

.history { display: flex; flex-direction: column; gap: 10px; }
.preview {
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}
.preview img { max-width: 100%; display: block; }
.preview.past { opacity: 0.4; }
.preview.past:hover { opacity: 0.85; transition: opacity 0.15s; }
.empty { min-height: 180px; color: #333; font-size: 0.875em; }

.status { display: block; margin-top: 10px; font-size: 0.72em; color: #333; }
</style>
