import { describe, expect, it } from 'vitest'

import { BRANLLY_FRAME_COUNT, nextFrame } from './animation'

describe('Branlly frame sequencing', () => {
  it('walks through frames and wraps without an out-of-range path', () => {
    expect(nextFrame(0)).toBe(1)
    expect(nextFrame(BRANLLY_FRAME_COUNT - 1)).toBe(0)
  })

  it('rejects corrupted animation state', () => {
    expect(() => nextFrame(-1)).toThrow(RangeError)
    expect(() => nextFrame(BRANLLY_FRAME_COUNT)).toThrow(RangeError)
    expect(() => nextFrame(0, 0)).toThrow(RangeError)
  })
})
