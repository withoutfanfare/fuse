import type { PullRequest, ReviewRule } from '../types'

export function usePromptBuilder() {
  function buildPrompt(pr: PullRequest, options?: { diff?: string; rules?: ReviewRule[] }): string {
    let prompt = `## Code Review Request\n\n`
    prompt += `### PR: #${pr.number} - ${pr.title}\n`
    prompt += `**Author:** ${pr.author}\n`
    prompt += `**Branch:** ${pr.head_branch} → ${pr.base_branch}\n`
    prompt += `**Changes:** +${pr.additions} / -${pr.deletions} across ${pr.changed_files} files\n`

    if (pr.labels.length > 0) {
      prompt += `**Labels:** ${pr.labels.join(', ')}\n`
    }

    prompt += '\n'

    if (options?.rules?.length) {
      prompt += `### Review Checklist\n`
      options.rules.forEach(r => {
        prompt += `- [ ] ${r.rule_text}\n`
      })
      prompt += '\n'
    }

    if (options?.diff) {
      const maxLength = 10000
      const truncatedDiff = options.diff.length > maxLength
        ? options.diff.substring(0, maxLength) + '\n\n... (truncated)'
        : options.diff
      prompt += `### Diff\n\`\`\`diff\n${truncatedDiff}\n\`\`\`\n\n`
    }

    prompt += `### Instructions\nPlease review this PR and provide:\n`
    prompt += `1. A brief summary of the changes\n`
    prompt += `2. Any issues found (critical, warning, or suggestion)\n`
    prompt += `3. Whether you would approve this PR\n`
    prompt += `\nFormat your response with clear sections.\n`

    return prompt
  }

  return { buildPrompt }
}
