<template>
  <main>
    <header>
      <NuxtLink to="/" class="back">← Printers</NuxtLink>
      <h1>{{ printer }}</h1>
    </header>

    <section class="endpoint">
      <span class="label">Printer Address</span>
      <div class="copy-row">
        <code>{{ endpoint }}</code>
        <button class="copy-btn" @click="copy">{{ copied ? '✓ Copied' : 'Copy' }}</button>
      </div>
    </section>

    <section class="config">
      <span class="label">Label size (inches)</span>
      <div class="config-row">
        <input v-model.number="width" type="number" step="0.25" min="0.5" max="12" />
        <span class="sep">×</span>
        <input v-model.number="height" type="number" step="0.25" min="0.5" max="12" />
        <span class="save-status">{{ configSaved ? '✓ Saved' : '' }}</span>
      </div>
    </section>

    <section class="history">
      <div v-if="labels.length === 0" class="preview empty-state">
        <div class="empty">Waiting for label…</div>
      </div>
      <div v-for="(src, i) in labels" :key="i" class="preview" :class="{ past: i > 0 }">
        <img :src="src" alt="Rendered label" />
      </div>
    </section>

    <span class="status">{{ status }}</span>
  </main>
</template>

<script setup lang="ts">
const route = useRoute()
const config = useRuntimeConfig()
const printer = route.params.printer as string

const apiBase = config.public.apiUrl || (process.client ? window.location.origin : '')
const endpoint = `${apiBase}/${printer}`.replace(/^https?:\/\//, '')

const labels = ref<string[]>([])
const status = ref('Connecting…')
const copied = ref(false)
const width = ref(2.25)
const height = ref(1.25)
const configSaved = ref(false)
let saveTimer: ReturnType<typeof setTimeout> | null = null

watch([width, height], () => {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(saveConfig, 600)
})

let es: EventSource
onMounted(async () => {
  const cfg = await fetch(`${apiBase}/${printer}/config`).then(r => r.json())
  width.value = cfg.width
  height.value = cfg.height

  es = new EventSource(`${apiBase}/${printer}/stream`)
  es.onopen = () => (status.value = '● Live')
  es.onerror = () => (status.value = 'Disconnected — retrying…')
  es.onmessage = (e) => {
    labels.value.unshift(`data:image/png;base64,${e.data}`)
    if (labels.value.length > 20) labels.value.pop()
    status.value = `Updated ${new Date().toLocaleTimeString()}`
  }
})
onUnmounted(() => es?.close())

function copy() {
  navigator.clipboard.writeText(endpoint)
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}

async function saveConfig() {
  await fetch(`${apiBase}/${printer}/config`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ width: width.value, height: height.value }),
  })
  configSaved.value = true
  setTimeout(() => (configSaved.value = false), 1500)
}
</script>

<style scoped>
main { max-width: 700px; margin: 48px auto; padding: 0 24px; }

header { display: flex; align-items: baseline; gap: 14px; margin-bottom: 28px; }
.back { font-size: 0.85em; color: #444; text-decoration: none; transition: color 0.15s; }
.back:hover { color: #aaa; }
h1 { margin: 0; font-size: 1.25em; font-family: monospace; font-weight: 600; }

.endpoint { margin-bottom: 20px; }
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
  background: #141414;
  border: 1px solid #222;
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

.config { margin-bottom: 20px; }
.config-row { display: flex; align-items: center; gap: 8px; }
.config-row input {
  width: 72px;
  background: #0d0d0d;
  border: 1px solid #2a2a2a;
  border-radius: 7px;
  padding: 9px 11px;
  color: #e0e0e0;
  font-size: 0.875em;
  text-align: center;
  outline: none;
}
.config-row input:focus { border-color: #444; }
.sep { color: #444; font-size: 0.9em; }
.save-status { font-size: 0.75em; color: #555; min-width: 48px; }

.history { display: flex; flex-direction: column; gap: 10px; }
.preview {
  background: #141414;
  border: 1px solid #222;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}
.empty-state { min-height: 180px; }
.preview img { max-width: 100%; display: block; }
.preview.past { opacity: 0.4; }
.preview.past:hover { opacity: 0.85; transition: opacity 0.15s; }
.empty { color: #333; font-size: 0.875em; }

.status { display: block; margin-top: 10px; font-size: 0.72em; color: #333; }
</style>