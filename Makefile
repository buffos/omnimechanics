SHELL := pwsh.exe
.SHELLFLAGS := -NoProfile -Command

# Ensure user-level tools (cargo, pnpm) are on PATH
export PATH := $(USERPROFILE)/.cargo/bin;$(APPDATA)/npm;$(PATH)

.DEFAULT_GOAL := help

null  :=
space := $(null) #

.PHONY: help info dev web build build-web build-rust test test-web test-rust lint lint-web lint-rust clean

help:
	@$(info )
	@$(info Available make commands for BemApp)
	@$(info ==================================)
	@$(info )
	@$(info Development:)
	@$(info $(space)$(space)dev               Start the Tauri v2 desktop application in development mode)
	@$(info $(space)$(space)web               Start the Angular web development server standalone)
	@$(info )
	@$(info Build:)
	@$(info $(space)$(space)build             Build production standalone desktop app)
	@$(info $(space)$(space)build-web         Build production Angular frontend)
	@$(info $(space)$(space)build-rust        Build production Rust binary (embedded frontend))
	@$(info )
	@$(info Quality & Verification:)
	@$(info $(space)$(space)test              Run all tests (frontend + rust))
	@$(info $(space)$(space)test-web          Run Angular frontend unit tests)
	@$(info $(space)$(space)test-rust         Run Rust unit & integration tests)
	@$(info $(space)$(space)lint              Run all linters & typechecks)
	@$(info $(space)$(space)lint-web          Run Angular build check)
	@$(info $(space)$(space)lint-rust         Run Rust cargo check)
	@$(info )
	@$(info Maintenance:)
	@$(info $(space)$(space)clean             Clean build outputs, dist, and cache)
	@$(info )

info: help

dev:
	pnpm run tauri dev

web:
	pnpm run start

build: build-web build-rust

build-web:
	pnpm run build

build-rust:
	cargo build --release --manifest-path src-tauri/Cargo.toml

test: test-rust test-web

test-rust:
	cargo test --manifest-path src-tauri/Cargo.toml

test-web:
	pnpm exec ng test --watch=false

lint: lint-web lint-rust

lint-web:
	pnpm run build

lint-rust:
	cargo check --manifest-path src-tauri/Cargo.toml

clean:
	if (Test-Path dist) { Remove-Item -Recurse -Force dist }; if (Test-Path src-tauri/target) { Remove-Item -Recurse -Force src-tauri/target }; if (Test-Path .angular) { Remove-Item -Recurse -Force .angular }
