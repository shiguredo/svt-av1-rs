.PHONY: test cover check clippy fmt clean

# 全テストを実行する
test:
	cargo test --features source-build

# 全テストカバレッジ付きで実行する
cover:
	cargo llvm-cov --tests --features source-build

# cargo check を実行する
check:
	cargo check --all-targets --features source-build

# cargo clippy を実行する
clippy:
	cargo clippy --all-targets --features source-build -- -D warnings

# cargo fmt を実行する
fmt:
	cargo fmt --all

# ビルド成果物を削除する
clean:
	cargo clean
