import type { ParsedAiResponse } from '../types'

export function useResponseParser() {
  function parseResponse(text: string): ParsedAiResponse {
    const lines = text.split('\n')
    const issues: ParsedAiResponse['issues'] = []
    let summary = ''
    let approved = false

    // Look for approval indicators
    const lowerText = text.toLowerCase()
    approved = lowerText.includes('approve') && !lowerText.includes('not approve') && !lowerText.includes("don't approve")

    // Extract summary (first paragraph or section after "summary")
    const summaryMatch = text.match(/(?:summary|overview)[:\s]*\n?([\s\S]*?)(?:\n#|\n\*\*|$)/i)
    summary = summaryMatch?.[1]?.trim() || lines.slice(0, 3).join(' ').trim()

    // Find issues
    for (const line of lines) {
      if (line.match(/critical|error|bug|vulnerability/i)) {
        issues.push({ severity: 'critical', description: line.replace(/^[-*•]\s*/, '').trim() })
      } else if (line.match(/warning|concern|should/i)) {
        issues.push({ severity: 'warning', description: line.replace(/^[-*•]\s*/, '').trim() })
      } else if (line.match(/suggestion|consider|minor|nit/i)) {
        issues.push({ severity: 'suggestion', description: line.replace(/^[-*•]\s*/, '').trim() })
      }
    }

    return { summary, issues, approved }
  }

  return { parseResponse }
}
