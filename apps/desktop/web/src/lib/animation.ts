export const BRANLLY_FRAME_COUNT = 32

export function nextFrame(current: number, frameCount = BRANLLY_FRAME_COUNT): number {
  if (!Number.isInteger(frameCount) || frameCount <= 0) {
    throw new RangeError('frameCount must be a positive integer')
  }
  if (!Number.isInteger(current) || current < 0 || current >= frameCount) {
    throw new RangeError('current frame is outside the animation range')
  }
  return (current + 1) % frameCount
}
