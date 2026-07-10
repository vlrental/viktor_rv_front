# AGENTS.md - Viktor RV Frontend

## SSH access

- Connect to the VL Rental server with `ssh vlrental`.
- SSH target: `root@159.203.47.33`, port `22`.
- The local SSH alias must use `~/.ssh/vlrental` with `IdentitiesOnly yes`.
- If the alias is unavailable, use `ssh -i ~/.ssh/vlrental root@159.203.47.33`.
- Never copy the private key into this repository, commits, logs, task descriptions, or chat output.
- Do not deploy, restart services, change production files, or execute production database writes over SSH without a direct user request.

## Safety

- Do not commit, push, deploy, or modify production without a direct user request.
- Check every changed text file for UTF-8/mojibake before completion.
