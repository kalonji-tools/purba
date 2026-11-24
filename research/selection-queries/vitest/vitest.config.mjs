import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['*.test.mjs'],
    tags: [
      { name: 'slow', description: 'a slow test' },
      { name: 'db', description: 'a test of the database' },
    ],
  },
})
