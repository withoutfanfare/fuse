/**
 * Composable that spawns a brief confetti burst of CSS-animated
 * particles from a given bounding rect. Used to celebrate merges.
 *
 * - 40–60 absolutely positioned div particles
 * - 1.5 s duration with gravity physics via CSS keyframes
 * - Brand colours (teal, violet, green, purple)
 * - Auto-removed after animation completes
 */
export function useConfetti() {
  const BRAND_COLOURS = [
    '#2dd4bf', // teal-400
    '#14b8a6', // teal-500
    '#a78bfa', // violet-400
    '#8b5cf6', // violet-500
    '#c084fc', // purple-400
    '#4ade80', // green-400
  ]

  const DURATION = 1500 // ms

  /**
   * Inject the keyframe stylesheet once (idempotent).
   */
  function ensureStyles() {
    if (document.getElementById('confetti-styles')) return
    const style = document.createElement('style')
    style.id = 'confetti-styles'
    style.textContent = `
      @keyframes confetti-fall {
        0% {
          transform: translate(var(--confetti-dx-start), 0) rotate(0deg);
          opacity: 1;
        }
        100% {
          transform: translate(var(--confetti-dx-end), var(--confetti-dy)) rotate(var(--confetti-rot));
          opacity: 0;
        }
      }
      .confetti-particle {
        position: fixed;
        pointer-events: none;
        z-index: 9999;
        animation: confetti-fall var(--confetti-duration) cubic-bezier(0.25, 0.46, 0.45, 0.94) forwards;
      }
    `
    document.head.appendChild(style)
  }

  /**
   * Random number between min and max (inclusive).
   */
  function rand(min: number, max: number): number {
    return Math.random() * (max - min) + min
  }

  /**
   * Fire confetti from the centre of a given bounding rect.
   */
  function fire(rect: DOMRect) {
    ensureStyles()

    const count = Math.floor(rand(40, 60))
    const centreX = rect.left + rect.width / 2
    const centreY = rect.top + rect.height / 2

    const container = document.createDocumentFragment()
    const particles: HTMLDivElement[] = []

    for (let i = 0; i < count; i++) {
      const el = document.createElement('div')
      el.className = 'confetti-particle'

      /* Random size — mix of rectangles and squares */
      const width = rand(4, 8)
      const height = rand(3, 10)
      el.style.width = `${width}px`
      el.style.height = `${height}px`
      el.style.borderRadius = `${rand(0, 2)}px`
      el.style.backgroundColor = BRAND_COLOURS[Math.floor(Math.random() * BRAND_COLOURS.length)]

      /* Position at centre of the source rect */
      el.style.left = `${centreX}px`
      el.style.top = `${centreY}px`

      /* Randomised physics via custom properties */
      const dxStart = rand(-40, 40)
      const dxEnd = dxStart + rand(-80, 80)
      const dy = rand(60, 220) // gravity — always falls downward
      const rotation = rand(-720, 720)
      const duration = rand(DURATION * 0.7, DURATION)

      el.style.setProperty('--confetti-dx-start', `${dxStart}px`)
      el.style.setProperty('--confetti-dx-end', `${dxEnd}px`)
      el.style.setProperty('--confetti-dy', `${dy}px`)
      el.style.setProperty('--confetti-rot', `${rotation}deg`)
      el.style.setProperty('--confetti-duration', `${duration}ms`)

      /* Staggered start for a more natural burst */
      el.style.animationDelay = `${rand(0, 100)}ms`

      particles.push(el)
      container.appendChild(el)
    }

    document.body.appendChild(container)

    /* Clean up after longest possible animation */
    setTimeout(() => {
      particles.forEach((p) => p.remove())
    }, DURATION + 200)
  }

  /**
   * Fire confetti from a given HTML element's position.
   */
  function fireFromElement(el: HTMLElement) {
    fire(el.getBoundingClientRect())
  }

  return { fire, fireFromElement }
}
