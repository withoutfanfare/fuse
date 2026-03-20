# PR Review Companion — Branding Ideas

## Overview
A Tauri menu bar application for intelligent, always-on pull request monitoring and AI-assisted code review. Lives in your macOS menu bar, always watching.

---

## Name Ideas

### Primary Recommendation
**Sentry** — Guardian, watchful, protective. Suggests always-on vigilance. Short, memorable, strong. (Note: May conflict with existing dev tool, consider alternatives below)

### Alternative Primary
**Vigil** — From vigilance, watchfulness. Elegant, suggests careful attention.

### Alternative Names

| Name | Rationale |
|------|-----------|
| ** lookout** | Watchful observer, spotter |
| **Warden** | Guardian, protector of code quality |
| **Beacon** | Signal, guide, warning light |
| **Sentinel** | Guard, watchman (similar to Sentry) |
| **Overwatch** | Observing from above, protective |
| **Guardian** | Protector, defender of quality |
| **Patrol** | Regular monitoring, checking |
| **Scout** | Reconnaissance, early warning |
| **Watchtower** | Elevated view, early detection |
| **Recon** | Reconnaissance, intelligence gathering |
| **Triage** | Sorting by priority, medical precision |
| **Brief** | Short summary, preparation |
| **Primer** | Preparation before action |
| **Cue** | Signal to act, prompt |
| **Nudge** | Gentle prompt, not intrusive |

---

## Tagline Options

### Primary Recommendations
> **"Always watching. Always ready."**

> **"Know before you review."**

### Alternative Taglines

- "Never review cold again"
- "Your PR briefing, delivered"
- "Review with confidence"
- "The intelligence you need, when you need it"
- "See what matters first"
- "Code review, briefed"
- "Your second pair of eyes, always on"
- "Review prepared"
- "Know the risk before you dive in"
- "Intelligent code review"
- "The briefing layer for GitHub"

---

## Visual Direction

### Mood & Aesthetic
**Always-on vigilance with understated intelligence**

The visual language should feel:
- **Always-on** — persistent but not intrusive
- **Intelligent** — smart analysis, not just data display
- **Fast** — quick glances, immediate understanding
- **Professional** — trustworthy, serious about code quality
- **Unobtrusive** — present without being demanding

### Visual References
- Menu bar utilities (Bartender, Vanilla, Stats)
- Security/ monitoring tools (Little Snitch, Objective-See)
- Productivity apps (Todoist, Things)
- Military/aviation HUD elements (clean, precise, information-dense)

### Logo Concepts

#### Concept 1: The Watchful Eye
A stylised eye or monitoring element — suggesting vigilance and observation. Abstracted, not literal.

#### Concept 2: The Radar/Signal
Concentric circles or signal waves — suggesting detection and monitoring. Technical, precise.

#### Concept 3: The Shield + Check
Protective shield combined with checkmark or code brackets. Safety and quality assurance.

#### Concept 4: The Menu Bar Badge
Embrace the menu bar location — the icon is a refined badge or indicator that feels native to macOS.

#### Concept 5: The Briefing Document
Document/page with intelligence highlights — representing the briefing concept.

---

## Colour Palette

### Primary Palette

| Role | Colour | Hex | Usage |
|------|--------|-----|-------|
| Primary | Signal Orange | `#f97316` | Alerts, high priority |
| Secondary | Steel Blue | `#475569` | Professional, calm |
| Accent | Success Green | `#22c55e` | Approved, low risk |
| Warning | Amber | `#eab308` | Medium risk, caution |
| Danger | Crimson | `#dc2626` | High risk, blocking |

### Extended Palette

| Role | Colour | Hex | Usage |
|------|--------|-----|-------|
| Background | Menu Bar | Varies | Native macOS appearance |
| Surface | Panel | `#1e293b` | Dark mode panels |
| Text Primary | White/Gray | `#f8fafc` | Primary text |
| Text Secondary | Muted | `#94a3b8` | Secondary text |
| Border | Subtle | `#334155` | Dividers, borders |

### Menu Bar Badge Colours

| State | Colour | Indication |
|-------|--------|------------|
| None | Gray | No PRs waiting |
| Normal | White/Blue | 1-3 PRs, routine |
| Attention | Amber | PRs waiting, moderate age |
| Urgent | Orange/Red | High risk or stale PRs |

### Rationale
The palette prioritises functional clarity. Orange/red for urgent attention, green for safe/approved. Steel blue provides professional grounding. Colours must work in both light and dark menu bar modes.

---

## Typography

### Primary Typeface: **SF Pro** (system font)
Native macOS feel, respects user preferences, fast rendering.

### Monospace: **SF Mono**
For code snippets, file paths, commit SHAs.

### Data/Numbers: **SF Pro Rounded** or **Tabular**
For risk scores, PR counts, timestamps — clear, readable numbers.

---

## Tone of Voice

### Core Attributes

| Attribute | Expression |
|-----------|------------|
| **Professional** | Serious about code quality |
| **Efficient** | Respects the reviewer's time |
| **Intelligent** | Insightful, not just informational |
| **Unobtrusive** | Present but not pushy |
| **Helpful** | Genuinely useful, not novelty |

### Voice Examples

**Instead of:** "The application monitors your repositories for new pull requests requiring review."
**Use:** "Knows when PRs need you. Tells you what you need to know."

**Instead of:** "AI-powered risk assessment and code analysis capabilities."
**Use:** "See the risk before you open the diff."

---

## Application Icon

### Menu Bar Icon Concept
A simple, recognisable silhouette optimised for 16-22px:
- Eye/watch element, or
- Shield/badge shape, or
- Document with signal indicator

Must be:
- Immediately recognisable
- Distinct from other menu bar icons
- Works in both light and dark modes
- Clear at tiny sizes

### App Icon (when visible)
A more detailed version for app switcher, about panel, etc.:
- Rounded rectangle with the menu bar icon centred
- Subtle gradient or depth
- Professional, tool-like appearance

---

## UI Patterns

### Menu Bar Popover
- Compact, information-dense
- PR list with key metadata
- Risk score badges
- One-click access to briefing
- "Open in GitHub" actions

### PR Briefing Panel
- Risk score prominently displayed
- AI analysis summary
- Custom checklist results
- File change summary
- Quick actions (approve, comment, etc.)

### Notification Style
- Native macOS notifications
- Brief, actionable
- Risk level indicated
- Direct link to briefing

---

## Feature Naming

| Feature | Branded Name | Rationale |
|---------|--------------|-----------|
| Menu bar badge | **Beacon** | Always visible signal |
| Risk scoring | **RiskIndex** | 1-10 risk assessment |
| AI diff analysis | **Insight** | Deep understanding |
| Custom checklists | **Rules** | Your project's requirements |
| PR briefing | **Brief** | Quick preparation |
| Smart notifications | **Alerts** | Intelligent filtering |
| Comment drafting | **Compose** | Write reviews |

---

## Risk Score Visual System

| Score | Colour | Label | Meaning |
|-------|--------|-------|---------|
| 1-3 | Green | Low | Safe to approve quickly |
| 4-6 | Yellow | Medium | Review with attention |
| 7-8 | Orange | High | Careful review needed |
| 9-10 | Red | Critical | Major concerns, block |

---

## Brand Architecture

### Product Hierarchy
```
Vigil (primary brand)
├── Vigil Desktop (menu bar app)
├── Vigil Teams (shared rules)
└── Vigil Enterprise (GitHub Enterprise support)
```

---

## Marketing Angles

### For Agency Tech Leads
"Eight repos. One view. Zero surprises."

### For Freelancers
"Review with confidence across all your clients."

### For CTOs
"Stay close to code quality without becoming a bottleneck."

---

## Competitive Differentiation

Unlike CodeRabbit or Copilot Review:
- **Reviewer-centric** — serves the reviewer, not the PR thread
- **Always-on** — ambient presence, not just when opened
- **Privacy-first** — local analysis, no code leaves your machine
- **Custom rules** — your domain expertise, enforced

---

## Personality Moments

### Empty States
- "All caught up. Time to ship."
- "No PRs waiting. Enjoy the calm."

### High Risk Alert
- "High-risk PR detected. Review carefully."
- "This one needs your full attention."

### Checklist Flag
- "Rules flagged: 2 items need review."

---

*Document created: 18 February 2026*
