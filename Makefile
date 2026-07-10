# VL Rental - Makefile (аренда RV/лодок). Курирован из idyll_v2, упрощён под web-only проект.
# Windows -> PowerShell, macOS/Linux -> bash. `dx serve` одинаков на всех ОС; отличается только kill.

PORT ?= 8080

UNAME_S := $(shell uname -s 2>/dev/null)
ifneq (,$(filter Darwin Linux,$(UNAME_S)))
  KILL_DEV = bash -c 'lsof -ti:$(PORT) | xargs -r kill -9 2>/dev/null; echo "port $(PORT) cleared"'
else
  KILL_DEV = powershell -ExecutionPolicy Bypass -File scripts/kill-dev.ps1
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
