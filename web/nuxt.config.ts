export default defineNuxtConfig({
  ssr: false,
  compatibilityDate: '2025-01-01',
  runtimeConfig: {
    public: {
      // Empty = same origin (Docker). Set API_URL for separate dev server.
      apiUrl: process.env.API_URL || '',
    },
  },
})