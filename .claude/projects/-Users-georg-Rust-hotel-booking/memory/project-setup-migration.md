---
name: project-setup-migration
description: Migration status from old .claude_old setup to new .claude agent/skills setup
metadata:
  type: project
---

Migration from old rust-skills-based setup (.claude_old, CLAUDE_OLD.md) to new agent+skills pipeline (.claude, CLAUDE.md) was done on 2026-05-15.

**Completed migrations:**
- `.claude/settings.json` created with project permission allowlist (ported from `.claude_old/settings.local.json`)
- `docs/adr/2026-05-06-country-codes-iso-3166-1-alpha-2.md` — ADR 001 migrated from `docs/decissions/`
- `docs/adr/2026-05-09-validation-at-handler-boundary.md` — ADR 002 migrated from `docs/decissions/`
- `docs/adr/README.md` — index updated with both ADRs
- `CLAUDE.md` — added Database Schema Changes section and references to dev_infos.md and implementation_plan.md
- `docs/implementation_plan.md` — marked as historical read-only document

**Still pending (not done — need human decision):**
1. ~~`docs/prd.md`~~ — DONE 2026-05-15 (22 requirements, 13 edge cases, 7 non-goals)
2. ~~`docs/system-design.md`~~ — DONE 2026-05-15 (15 sections, real flat-layer layout, all request flows documented)
3. `docs/decissions/` old directory — still exists alongside new `docs/adr/`; can be deleted once ADRs are verified
4. `CLAUDE_OLD.md` and `.claude_old/` — still exist; safe to delete once verified nothing is missing

**How to apply:** When starting new features, remind user that docs/prd.md and docs/system-design.md are still empty templates and need to be filled before the agent pipeline will work properly.
