
import type { GlobalThemeOverrides } from 'naive-ui'

declare global {
  interface Window {
    $naive?: {
      theme?: GlobalThemeOverrides
    }
  }
}

export {}