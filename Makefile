.PHONY: help install build build-release test lint clean run install-local uninstall-local

help: ## ヘルプを表示
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## 依存関係をインストール
	rustup target add x86_64-apple-darwin aarch64-apple-darwin

build: ## デバッグビルド
	cargo build

build-release: ## リリースビルド（ユニバーサルバイナリ）
	cargo build --release --target x86_64-apple-darwin
	cargo build --release --target aarch64-apple-darwin
	mkdir -p target/release
	lipo -create \
		target/x86_64-apple-darwin/release/pomodoro \
		target/aarch64-apple-darwin/release/pomodoro \
		-output target/release/pomodoro-universal

test: ## テスト実行
	cargo test --all-features

lint: ## Lint実行
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

clean: ## ビルド成果物削除
	cargo clean

run: ## デーモン起動
	cargo run -- daemon

install-local: build-release ## ローカルにインストール
	sudo cp target/release/pomodoro-universal /usr/local/bin/pomodoro
	sudo chmod +x /usr/local/bin/pomodoro

uninstall-local: ## ローカルからアンインストール
	sudo rm -f /usr/local/bin/pomodoro
