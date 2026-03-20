import { ref, shallowRef } from 'vue'

/**
 * Language-to-highlight.js grammar mapping based on file extension.
 * Only the most common languages used in PR reviews are included.
 */
const EXTENSION_MAP: Record<string, string> = {
  ts: 'typescript',
  tsx: 'typescript',
  js: 'javascript',
  jsx: 'javascript',
  mjs: 'javascript',
  cjs: 'javascript',
  vue: 'xml',
  svelte: 'xml',
  php: 'php',
  rs: 'rust',
  go: 'go',
  py: 'python',
  rb: 'ruby',
  java: 'java',
  kt: 'kotlin',
  swift: 'swift',
  cs: 'csharp',
  css: 'css',
  scss: 'scss',
  less: 'less',
  html: 'xml',
  xml: 'xml',
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  md: 'markdown',
  mdx: 'markdown',
  toml: 'ini',
  sql: 'sql',
  sh: 'bash',
  bash: 'bash',
  zsh: 'bash',
  dockerfile: 'dockerfile',
  graphql: 'graphql',
  gql: 'graphql',
}

/**
 * Composable for lazy-loading highlight.js and applying syntax highlighting
 * to diff content. The library is only loaded when first needed.
 */
export function useSyntaxHighlight() {
  const hljs = shallowRef<any>(null)
  const loading = ref(false)
  const loaded = ref(false)

  /**
   * Lazily load highlight.js and core languages.
   */
  async function ensureLoaded(): Promise<boolean> {
    if (loaded.value) return true
    if (loading.value) return false

    loading.value = true
    try {
      // Dynamic import — highlight.js is only loaded when syntax highlighting is first used
      const module = await import('highlight.js/lib/core')
      const hljsCore = module.default

      // Register common languages
      const [
        typescript,
        javascript,
        xml,
        css,
        json,
        yaml,
        markdown,
        rust,
        python,
        go,
        php,
        sql,
        bash,
      ] = await Promise.all([
        import('highlight.js/lib/languages/typescript'),
        import('highlight.js/lib/languages/javascript'),
        import('highlight.js/lib/languages/xml'),
        import('highlight.js/lib/languages/css'),
        import('highlight.js/lib/languages/json'),
        import('highlight.js/lib/languages/yaml'),
        import('highlight.js/lib/languages/markdown'),
        import('highlight.js/lib/languages/rust'),
        import('highlight.js/lib/languages/python'),
        import('highlight.js/lib/languages/go'),
        import('highlight.js/lib/languages/php'),
        import('highlight.js/lib/languages/sql'),
        import('highlight.js/lib/languages/bash'),
      ])

      hljsCore.registerLanguage('typescript', typescript.default)
      hljsCore.registerLanguage('javascript', javascript.default)
      hljsCore.registerLanguage('xml', xml.default)
      hljsCore.registerLanguage('css', css.default)
      hljsCore.registerLanguage('json', json.default)
      hljsCore.registerLanguage('yaml', yaml.default)
      hljsCore.registerLanguage('markdown', markdown.default)
      hljsCore.registerLanguage('rust', rust.default)
      hljsCore.registerLanguage('python', python.default)
      hljsCore.registerLanguage('go', go.default)
      hljsCore.registerLanguage('php', php.default)
      hljsCore.registerLanguage('sql', sql.default)
      hljsCore.registerLanguage('bash', bash.default)

      hljs.value = hljsCore
      loaded.value = true
      return true
    } catch {
      // If highlight.js fails to load, gracefully degrade — no highlighting
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * Detect the highlight.js language name from a file path's extension.
   */
  function detectLanguage(filePath: string): string | null {
    const ext = filePath.split('.').pop()?.toLowerCase()
    if (!ext) return null
    return EXTENSION_MAP[ext] ?? null
  }

  /**
   * Highlight a single line of code content. Returns HTML string with
   * syntax-highlighted spans, or the original text if no language is detected.
   */
  function highlightLine(content: string, language: string | null): string {
    if (!hljs.value || !language) return escapeHtml(content)

    try {
      const result = hljs.value.highlight(content, {
        language,
        ignoreIllegals: true,
      })
      return result.value
    } catch {
      return escapeHtml(content)
    }
  }

  return {
    loading,
    loaded,
    ensureLoaded,
    detectLanguage,
    highlightLine,
  }
}

/** Escape HTML entities for safe insertion. */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}
