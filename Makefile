# VL Rental - Makefile (аренда RV/лодок). Курирован из idyll_v2, упрощён под web-only проект.
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
	dx build --release

# CI deploy: push the current commit to main. GitHub Actions builds and
# publishes the site to GitHub Pages. The frontend has no database migration,
# so d and dm intentionally do the same thing.
deploy:
	git fetch origin && git push origin HEAD:refs/heads/main

deploy-migrate: deploy
	@echo "Frontend has no database migrations; deploy triggered."

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

.PHONY: dev devo build deploy deploy-migrate k cc c co b d dm
