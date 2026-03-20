# Fuse MVP

Tauri desktop MVP for reviewer-first PR triage with local risk scoring and AI briefing generation.

## What this MVP does

- Captures pull request entries into a local triage queue.
- Calculates per-PR risk score based on:
  - files changed
  - lines changed
  - PR age
  - test coverage presence
  - auth/database touchpoints
- Shows a menu-bar style pending PR badge with urgency states.
- Stores repo-specific review checklist rules.
- Builds an AI briefing pack for the selected PR.
- Supports marking items as approved.

## How to run

```bash
cd /Users/dannyharding/Development/Code/Project/fuse/tauri-app
npm install
npm run tauri dev
```

## How to use

1. Enter PR metadata and click `Add to Queue`.
2. Add repository checklist rules and save them.
3. Use the triage table to select a PR with `Brief`.
4. Review generated briefing text.
5. Use `Copy Briefing` to hand off into your AI workflow.
6. Mark complete with `Approve`.

## Local data storage

- SQLite database: `pr_companion.db`
- Stored in the app data directory managed by Tauri
- Contains tracked repositories, pull requests, review rules, and app settings

## MVP boundaries

- No live GitHub API polling yet.
- No actual menu bar process in this iteration; badge behaviour is simulated in the main UI.
- No automatic diff parsing yet; briefing content is based on entered metadata.
