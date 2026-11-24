import { describe, expect, test } from 'vitest'

describe('store', () => {
  test('query', { tags: ['db'] }, () => {
    expect(true, 'query').toBe(true)
  })
  test('migrate', { tags: ['slow', 'db'] }, () => {
    expect(true, 'migrate').toBe(true)
  })
})
