# BB-TG-022 — The audit had no rule for agent skill trees

id: BB-TG-022
status: FIXED
for: none
date: 2026-11-09

## Gap

The backback skill family (7 skills, a `cca-` and a `cda-` variant each)
moved into the repo. Claude Code and Codex find a skill only at
`.claude/skills/<name>/SKILL.md` and `.agents/skills/<name>/SKILL.md`. The
audit walks hidden folders, so both `skills/` folders failed the five-entry
cap (7 entries) and the `bb_` marker rule (a tool-fixed name cannot carry it).

## Workaround

`bb_tree::AGENT_TREES` lists the root `.claude` and `.agents` trees: the walk
skips the folder rules there and keeps the line caps, so every `SKILL.md`
stays within 100 lines. Test
`agent_skill_trees_keep_their_tool_layout_but_not_their_line_cap` in
`bb_crates/shell/audit/tests/tree_rules.rs` also proves a `docs/skills/`
folder is not exempt.
