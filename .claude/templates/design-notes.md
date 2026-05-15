# Design Notes: {{FEATURE_NAME}}

Feature: {{FEATURE_NAME}}
Requirement: {{REQUIREMENT_ID}}
Date: {{DATE}}
Status: APPROVED | NEEDS_CHANGES | BLOCKED

## Architectural Fit

{{Does this feature fit the existing module structure and Clean Architecture layers? Reference docs/system-design.md sections.}}

## Module Placement

| Component | Location | Notes |
|-----------|----------|-------|
| {{type/trait/fn}} | {{crate/module path}} | {{notes}} |

## Integration Points

{{Which existing modules, types, and traits are touched or extended?}}

## Patterns to Follow

{{Which patterns from docs/system-design.md apply? Reference section names.}}

## Failure Modes

{{What can go wrong? How should each case be handled? Map to error types.}}

## Risks

{{Technical risks, edge cases, constraints, open questions.}}

## Recommendation

Status: APPROVED | NEEDS_CHANGES | BLOCKED

{{Reasoning. If NEEDS_CHANGES or BLOCKED, list required changes before feature-implementer may proceed.}}
