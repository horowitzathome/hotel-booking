# Review Summary: {{FEATURE_NAME}}

Feature: {{FEATURE_NAME}}
Requirement: {{REQUIREMENT_ID}}
Date: {{DATE}}
Status: APPROVED | NEEDS_CHANGES | BLOCKED

## Reviewer Results

| Reviewer | Status | Findings |
|----------|--------|----------|
| code-quality-reviewer | {{status}} | {{count}} |
| test-reviewer | {{status}} | {{count}} |
| security-reviewer | {{status}} | {{count}} |
| doc-reviewer | {{status}} | {{count}} |

## Actions Taken

### Fixed ([AUTOFIX] and [BLOCKED])

<!-- List each fix applied. -->

- {{location}}: {{what was fixed}}

### Escalated

<!-- Items written to .scratch/escalations.md. -->

- {{ESC-NNN}}: {{brief description}}

### Clarifications Requested

<!-- Items routed to other agents for clarification. -->

- {{[CLARIFY:agent]}}: {{brief description}} → sent to {{agent}}

## Overall Decision

Status: APPROVED | NEEDS_CHANGES | BLOCKED

{{One sentence. If NEEDS_CHANGES: re-run quality gate and re-invoke reviewers after fixes. If APPROVED: feature is complete.}}
