# Review: {{REVIEW_TYPE}}

Feature: {{FEATURE_NAME}}
Requirement: {{REQUIREMENT_ID}}
Reviewer: {{REVIEWER_AGENT}}
Date: {{DATE}}
Status: APPROVED | NEEDS_CHANGES | BLOCKED

## Summary

{{One-sentence overall assessment.}}

## Findings

<!-- One entry per issue. Delete this section if Status: APPROVED with no findings. -->

### {{file/location}} — {{brief description}}
Tag: [AUTOFIX|BLOCKED|ESCALATE|CLARIFY:agent]
Severity: Critical | High | Medium | Low
Detail: {{Explanation and specific fix guidance.}}

## Build Output

<!-- Paste relevant cargo output here (build errors, clippy warnings, test failures). -->

```
{{cargo output}}
```
