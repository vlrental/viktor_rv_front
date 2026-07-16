# VL Rental - Makefile (RV-only). Курирован из idyll_v2, упрощён под web-only проект.
# Windows -> PowerShell, macOS/Linux -> bash. `dx serve` одинаков на всех ОС; отличается только kill.

PORT ?= 8080

ifeq ($(OS),Windows_NT)
  KILL_DEV = powershell -ExecutionPolicy Bypass -File scripts/kill-dev.ps1
else
  KILL_DEV = bash -c 'lsof -ti:$(PORT) | xargs -r kill -9 2>/dev/null; echo "port $(PORT) cleared"'
endif

# --- Разработка ---

# Основная команда: dev-сервер Dioxus с hot-reload на порту 8080.
dev:
	dx serve --platform web --port $(PORT) --hot-reload true

# То же + автооткрытие браузера.
devo:
	dx serve --platform web --port $(PORT) --hot-reload true --open true

# Прод-сборка (release).
build:
	dx build --release --web --debug-symbols false

# Push development work. This does not publish GitHub Pages.
deploy:
	@test "$$(git branch --show-current)" = "dev" || (echo "Run this command from the dev branch" && exit 1)
	git push origin dev

# Manually publish dev to the repository's test GitHub Pages URL. This workflow
# never adds a custom domain; run it only after the user directly requests it.
deploy-test:
	@test "$$(git branch --show-current)" = "dev" || (echo "Run this command from the dev branch" && exit 1)
	gh workflow run pages.yml --ref dev

# Promote the tested dev commit to production main. Only main pushes trigger the
# production Pages workflow automatically.
deploy-migrate:
	@test "$$(git branch --show-current)" = "dev" || (echo "Run this command from the dev branch" && exit 1)
	git fetch origin
	git push origin dev
	git push origin dev:main
	@echo "Frontend dev promoted to main; production deploy triggered."

# Убить зависший dev-сервер на порту 8080 (dx на других портах не трогает).
k:
	@$(KILL_DEV)

# Запустить Claude Code.
cc:
	claude

# --- Алиасы (как в idyll) ---
c: dev
co: devo
b: build
d: deploy
dm: deploy-migrate

.PHONY: dev devo build deploy deploy-test deploy-migrate k cc c co b d dm
