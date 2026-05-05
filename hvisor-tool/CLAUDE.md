# hvisor-tool

User-space tooling for the hvisor hypervisor. See `~/.claude/skills/cross-repo/SKILL.md` for cross-repo access patterns.

## Cross-Repo Context

This project is part of a multi-repo ecosystem. Other repos to access via absolute paths:

| Repo | Path |
|------|------|
| hvisor (hypervisor) | `/home/clk/workspace/hvisor-deploy/hvisor` |
| Linux kernel | `/home/clk/workspace/kernel` |
| Platform configs | `/home/clk/workspace/hvisor-deploy/platform` |
| Images | `/home/clk/workspace/hvisor-deploy/images` |

When asked about code outside this repo, read from the absolute paths above. Search across all repos with `grep -rn <pattern> --include="*.c" --include="*.h" --include="*.rs" /home/clk/workspace/hvisor-deploy/hvisor/ /home/clk/workspace/hvisor-deploy/hvisor-tool/ /home/clk/workspace/kernel/`.
