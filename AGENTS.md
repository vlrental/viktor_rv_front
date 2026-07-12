# AGENTS.md - Viktor RV Frontend

## Local project layout

- Frontend repository: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_front`.
- Backend repository: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_back`.
- Treat these as the paired frontend and backend repositories for the Viktor RV project.
- Keep shared project context and cross-repository working rules synchronized in both repositories' `AGENTS.md` files. When adding or changing a rule that applies to the whole project, write it to both the frontend and backend instructions.
- The frontend is the default working directory for this workspace. When a task involves API behavior, server code, routes, database access, or backend integrations, inspect the backend repository at the path above instead of searching for it.
- The user commonly runs both projects locally. Verify the actual processes and ports when runtime state matters; do not assume that a service is currently running solely from this note.

## Project design

- The canonical project design is available through the connected design tool. Open and inspect it directly when design context is needed; do not ask the user to provide it again unless access fails.
- Design Node IDs: `IUHnT`, `x8t0A0`, `I6W2Es`, `raX6S`, `rmfa4`, `ODW3r`, `f1GuCf`, `ns0xG`, `LaBip`, `iKgTN`, `Oijpd`, `MEsd0`, `XgqBg`, `w19Mf`, `jr5XP`, `CdnCR`, `X9ejnB`, `g9upP`, `qaZRF`, `yTWGi`, `Al6fI`, `lsQAl`, `eb6Ck`, `cOu0u`, `YuNUS`, `TDhXo`, `M4DJcJ`, `e8z6o4`, `K7A9o`.
- Use these nodes as the visual source of truth when implementing or reviewing the corresponding frontend UI.

## Application roles

- Application roles are stored in the backend `app_users.role` column and are limited to `default` and `admin`.
- New email/password and Google users receive the `default` role. Administrative access is granted only by changing the database role to `admin`; there is no standalone admin API token.

## Git branches and deployment

- Develop only on the `dev` branch. Push ordinary work to `origin/dev`; do not develop directly on `main`.
- `main` is the production branch. Promote `dev` to `main` only through `make dm` after relevant checks pass and the user directly requests deployment.
- `make d` pushes `dev` without deploying production. `make dm` pushes `dev`, promotes the same commit to `main`, and triggers the production GitHub Pages workflow.

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
