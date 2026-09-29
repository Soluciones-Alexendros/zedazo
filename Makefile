.PHONY: all check test build release clean hooks version bump fmt clippy doc deny audit help docs-validate parity web-ci

CARGO := cargo
BINARY := target/release/zedazo
PNPM := pnpm --dir apps/web

##@ Desarrollo

all: check test build

check: ## Analiza el código sin compilar binario final
	$(CARGO) check --workspace --all-features

test: ## Ejecuta todos los tests
	$(CARGO) test --workspace --all-features

parity: ## Tests de equivalencia CLI↔API HTTP (O10)
	$(CARGO) test -p zedazo-api --test equivalence_http --all-features

web-ci: ## Tokens check/contraste + lint + typecheck + unit tests + build + Playwright e2e/a11y/visual de apps/web
	CI=true $(PNPM) install --frozen-lockfile || CI=true $(PNPM) install
	cd apps/web && ./node_modules/.bin/tsc --noEmit
	cd apps/web && ./node_modules/.bin/next lint
	cd apps/web && node scripts/design-tokens/build.mjs --check
	cd apps/web && node scripts/design-tokens/contrast.mjs
	cd apps/web && node --test src/lib/api.test.mjs src/lib/design-tokens-contrast.test.mjs src/lib/atoms-contract.test.mjs src/lib/local-fixtures.test.mjs
	cd apps/web && ./node_modules/.bin/next build
	cd apps/web && ./node_modules/.bin/playwright install --with-deps chromium
	cd apps/web && ./node_modules/.bin/playwright test --grep-invert screenshots

build: ## Compila en modo debug
	$(CARGO) build --workspace

release: ## Compila en modo release
	$(CARGO) build -p zedazo --release --locked

clean: ## Limpia artefactos de compilación
	$(CARGO) clean

##@ Calidad

deny: ## Licencias/advisories (cargo-deny; ADR-0013)
	@command -v cargo-deny >/dev/null || (echo "Instala: cargo install cargo-deny" && exit 1)
	cargo deny check

audit: ## Advisories RustSec (cargo-audit; política en .cargo/audit.toml)
	@command -v cargo-audit >/dev/null || (echo "Instala: cargo install cargo-audit" && exit 1)
	cargo audit

docs-validate: ## Valida documentación canónica (frontmatter, enlaces, stubs, trazabilidad)
	@echo "→ Validando frontmatter en docs canónicos..."
	@for f in SPECS.md ROADMAP.md DECISIONS.md ARCHITECTURE.md AGENTS.md; do \
		if ! head -20 "$$f" | grep -q "^version:"; then \
			echo "❌ $$f: falta frontmatter"; exit 1; \
		fi; \
		echo "✓ $$f"; \
	done
	@echo "→ Validando stubs en docs/..."
	@for f in docs/spec.md docs/architecture.md docs/roadmap.md docs/tasks.md; do \
		if ! grep -q "redirige aquí\|canónico" "$$f"; then \
			echo "❌ $$f: no redirige a doc canónico"; exit 1; \
		fi; \
		echo "✓ $$f"; \
	done
	@echo "→ Validando enlaces internos..."
	@grep -r '\[.*\](\.?/.*\.md)' --include='*.md' . | grep -v '^Binary' | while read line; do \
		file=$$(echo "$$line" | cut -d: -f1); \
		link=$$(echo "$$line" | grep -o '](.*)' | sed 's/]//;s/(//;s/)//'); \
		if [[ "$$link" == http* ]]; then continue; fi; \
		target=$$(dirname "$$file")/$$link; \
		if [[ ! -f "$$target" ]]; then \
			echo "❌ Enlace roto en $$file: $$link"; exit 1; \
		fi; \
	done
	@echo "✓ Enlaces internos OK"
	@echo "→ Validando trazabilidad SPECS↔ROADMAP↔DECISIONS..."
	@if ! grep -q "SPECS.md" ROADMAP.md || ! grep -q "DECISIONS.md" ROADMAP.md; then \
		echo "❌ ROADMAP.md no referencia SPECS/DECISIONS"; exit 1; \
	fi
	@if ! grep -q "ROADMAP.md" SPECS.md || ! grep -q "DECISIONS.md" SPECS.md; then \
		echo "❌ SPECS.md no referencia ROADMAP/DECISIONS"; exit 1; \
	fi
	@if ! grep -q "SPECS.md" DECISIONS.md || ! grep -q "ROADMAP.md" DECISIONS.md; then \
		echo "❌ DECISIONS.md no referencia SPECS/ROADMAP"; exit 1; \
	fi
	@if ! grep -q "ROADMAP.md" ARCHITECTURE.md || ! grep -q "DECISIONS.md" ARCHITECTURE.md; then \
		echo "❌ ARCHITECTURE.md no referencia ROADMAP/DECISIONS"; exit 1; \
	fi
	@echo "✓ Trazabilidad OK"
	@echo "→ Validando landing de producto..."
	@test -f apps/landing/index.html || (echo "❌ falta apps/landing/index.html"; exit 1)
	@test -f apps/landing/favicon.svg || (echo "❌ falta apps/landing/favicon.svg"; exit 1)
	@test -f deploy/Caddyfile.landing || (echo "❌ falta deploy/Caddyfile.landing"; exit 1)
	@grep -q "https://github.com/Iniciativas-Alexendros/zedazo" apps/landing/index.html || (echo "❌ landing: falta enlace GitHub"; exit 1)
	@grep -q "https://crates.io/crates/zedazo" apps/landing/index.html || (echo "❌ landing: falta enlace crates.io"; exit 1)
	@grep -q "https://docs.rs/zedazo" apps/landing/index.html || (echo "❌ landing: falta enlace docs.rs"; exit 1)
	@grep -q "zedazo.alexendros.dev" apps/landing/index.html || (echo "❌ landing: falta wordmark de dominio"; exit 1)
	@grep -q "zedazo.alexendros.dev" deploy/Caddyfile.landing || (echo "❌ Caddyfile.landing: falta hostname de producto"; exit 1)
	@grep -Eq 'class="wordmark">zedazo</(span|h1)>' apps/landing/index.html || (echo "❌ landing: wordmark no está en minúsculas"; exit 1)
	@grep -q 'href="./tokens.css"' apps/landing/index.html || (echo "❌ landing: no consume tokens.css generado"; exit 1)
	@test -f apps/landing/tokens.css || (echo "❌ falta apps/landing/tokens.css (pnpm tokens:build)"; exit 1)
	@grep -q 'ZEDAZO_WORDMARK = "zedazo"' apps/web/src/components/brand/zedazo-wordmark.tsx || (echo "❌ GUI: wordmark no está en minúsculas"; exit 1)
	@test -f docs/brand.md || (echo "❌ falta docs/brand.md"; exit 1)
	@echo "✓ Landing OK"
	@echo "✓ docs-validate completado"

traceability: ## Genera matriz de trazabilidad SPECS→Módulo→Test
	@bash scripts/traceability.sh

fmt: ## Formatea el código
	$(CARGO) fmt --all

fmt-check: ## Verifica el formateo (CI)
	$(CARGO) fmt --all -- --check

clippy: ## Linter estricto
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

lint: fmt clippy ## Formatea + linter

##@ Git

hooks: ## Instala hooks pre-commit
	@sh .githooks/install.sh

##@ Versionado

version: ## Muestra la versión actual desde Cargo.toml
	@grep '^version' crates/zedazo-cli/Cargo.toml 2>/dev/null | head -1 || grep 'version.workspace' -A0 crates/zedazo-cli/Cargo.toml; grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'

bump-patch: ## Incrementa versión patch (0.1.0 → 0.1.1)
	@sh scripts/bump.sh patch

bump-minor: ## Incrementa versión minor (0.1.0 → 0.2.0)
	@sh scripts/bump.sh minor

bump-major: ## Incrementa versión major (0.1.0 → 1.0.0)
	@sh scripts/bump.sh major

##@ CI local

ci: fmt-check clippy test check doc docs-validate parity web-ci ## Simula CI completa (Rust + O10 + web)

doc: ## Genera documentación
	$(CARGO) doc --workspace --no-deps --document-private-items
	@echo "Documentación generada en target/doc/"

##@ Otros

completions: ## Genera scripts de autocompletado (bash/zsh/fish)
	$(CARGO) build -p zedazo --release
	mkdir -p completions
	$(BINARY) completions bash > completions/zedazo.bash
	$(BINARY) completions zsh  > completions/_zedazo
	$(BINARY) completions fish > completions/zedazo.fish
	@echo "Completions generados en completions/"

help: ## Muestra esta ayuda
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\t\033[0m %s\n", $$1, $$2}'
