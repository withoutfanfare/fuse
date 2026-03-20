import { marked } from 'marked'
import DOMPurify from 'dompurify'

export function useMarkdown() {
  function renderMarkdown(text: string): string {
    const html = marked.parse(text, { breaks: true, gfm: true }) as string
    return DOMPurify.sanitize(html)
  }

  return { renderMarkdown }
}
