import { describe, expect, test } from 'vitest'

describe('math', () => {
  test('add', () => {
    expect(1 + 1, 'addition').toBe(2)
  })
  test('add slow', { tags: ['slow'] }, () => {
    expect(2 + 2, 'addition, slow').toBe(4)
  })
})
