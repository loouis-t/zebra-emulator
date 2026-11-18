<template>
  <NuxtPage />
</template>

<style>
*, *::before, *::after { box-sizing: border-box; }
body { margin: 0; background: #0d0d0d; color: #e0e0e0; font-family: system-ui, -apple-system, sans-serif; }
.surface { background: #141414; border: 1px solid #222; }
input {
  background: #0d0d0d;
  border: 1px solid #2a2a2a;
  border-radius: 7px;
  color: #e0e0e0;
  outline: none;
  transition: border-color 0.15s;
}
input:focus { border-color: #444; }
</style>
