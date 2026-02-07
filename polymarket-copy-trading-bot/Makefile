TARGET_X86_64 = x86_64-pc-windows-gnu
TARGET_I686 = i686-pc-windows-gnu
PROJECT_NAME = polymarket-copy-rust
CARGO = cargo

.PHONY: help
help:
	@$(CARGO) run --release --bin help 2>/dev/null || $(CARGO) run --bin help

.PHONY: setup
setup:
	@$(CARGO) run --release --bin setup 2>/dev/null || $(CARGO) run --bin setup

.PHONY: validate-setup
validate-setup:
	@$(CARGO) run --release --bin validate_setup 2>/dev/null || $(CARGO) run --bin validate_setup

.PHONY: health-check
health-check:
	@$(CARGO) run --release --bin health_check 2>/dev/null || $(CARGO) run --bin health_check

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: run
run:
	@if [ ! -f .env ]; then \
		echo "[ERROR] .env file not found!"; \
		echo "Run: make setup"; \
		exit 1; \
	fi
	@$(CARGO) run --release --bin validate_setup || exit 1
	@$(CARGO) build --release
	@$(CARGO) run --release

.PHONY: dev
dev:
	$(CARGO) run

# -----------------------------------------------------------------------------
# Wallet & allowance
# -----------------------------------------------------------------------------
.PHONY: check-allowance
check-allowance:
	@$(CARGO) run --release --bin check_allowance 2>/dev/null || $(CARGO) run --bin check_allowance

.PHONY: verify-allowance
verify-allowance:
	@$(CARGO) run --release --bin verify_allowance 2>/dev/null || $(CARGO) run --bin verify_allowance

.PHONY: set-token-allowance
set-token-allowance:
	@$(CARGO) run --release --bin set_token_allowance 2>/dev/null || $(CARGO) run --bin set_token_allowance

.PHONY: check-proxy
check-proxy:
	@$(CARGO) run --release --bin check_proxy 2>/dev/null || $(CARGO) run --bin check_proxy

.PHONY: check-both
check-both:
	@$(CARGO) run --release --bin check_both 2>/dev/null || $(CARGO) run --bin check_both

.PHONY: check-stats
check-stats:
	@$(CARGO) run --release --bin check_stats 2>/dev/null || $(CARGO) run --bin check_stats

.PHONY: check-activity
check-activity:
	@$(CARGO) run --release --bin check_activity 2>/dev/null || $(CARGO) run --bin check_activity

.PHONY: check-pnl
check-pnl:
	@$(CARGO) run --release --bin check_pnl 2>/dev/null || $(CARGO) run --bin check_pnl

.PHONY: manual-sell
manual-sell:
	@$(CARGO) run --release --bin manual_sell 2>/dev/null || $(CARGO) run --bin manual_sell

.PHONY: sell-large
sell-large:
	@$(CARGO) run --release --bin sell_large 2>/dev/null || $(CARGO) run --bin sell_large

.PHONY: close-stale
close-stale:
	@$(CARGO) run --release --bin close_stale 2>/dev/null || $(CARGO) run --bin close_stale

.PHONY: close-resolved
close-resolved:
	@$(CARGO) run --release --bin close_resolved 2>/dev/null || $(CARGO) run --bin close_resolved

.PHONY: redeem-resolved
redeem-resolved:
	@$(CARGO) run --release --bin redeem_resolved 2>/dev/null || $(CARGO) run --bin redeem_resolved

.PHONY: transfer-to-gnosis
transfer-to-gnosis:
	@$(CARGO) run --release --bin transfer_to_gnosis 2>/dev/null || $(CARGO) run --bin transfer_to_gnosis

.PHONY: find-traders
find-traders:
	@$(CARGO) run --release --bin find_traders 2>/dev/null || $(CARGO) run --bin find_traders

.PHONY: find-low-risk
find-low-risk:
	@$(CARGO) run --release --bin find_low_risk 2>/dev/null || $(CARGO) run --bin find_low_risk

.PHONY: scan-traders
scan-traders:
	@$(CARGO) run --release --bin scan_traders 2>/dev/null || $(CARGO) run --bin scan_traders

.PHONY: scan-markets
scan-markets:
	@$(CARGO) run --release --bin scan_markets 2>/dev/null || $(CARGO) run --bin scan_markets

.PHONY: simulate
simulate:
	@$(CARGO) run --release --bin simulate 2>/dev/null || $(CARGO) run --bin simulate

.PHONY: simulate-old
simulate-old:
	@$(CARGO) run --release --bin simulate_old 2>/dev/null || $(CARGO) run --bin simulate_old

.PHONY: sim
sim:
	@$(CARGO) run --release --bin sim 2>/dev/null || $(CARGO) run --bin sim

.PHONY: compare
compare:
	@$(CARGO) run --release --bin compare 2>/dev/null || $(CARGO) run --bin compare

.PHONY: fetch-history
fetch-history:
	@$(CARGO) run --release --bin fetch_history 2>/dev/null || $(CARGO) run --bin fetch_history

.PHONY: aggregate
aggregate:
	@$(CARGO) run --release --bin aggregate 2>/dev/null || $(CARGO) run --bin aggregate

.PHONY: audit
audit:
	@$(CARGO) run --release --bin audit 2>/dev/null || $(CARGO) run --bin audit

.PHONY: audit-old
audit-old:
	@$(CARGO) run --release --bin audit_old 2>/dev/null || $(CARGO) run --bin audit_old

.PHONY: default
default: run

.PHONY: clean
clean:
	$(CARGO) clean

.PHONY: install
install:
	sudo apt update
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	sudo apt install -y mingw-w64 2>/dev/null || true
	rustup target add $(TARGET_X86_64) 2>/dev/null || true
	rustup target add $(TARGET_I686) 2>/dev/null || true

.PHONY: pm2
pm2: build
	pm2 start target/release/$(PROJECT_NAME) --name polymarket-bot

.PHONY: pm2-telegram-bot
pm2-telegram-bot: build
	pm2 start bash --name telegram-bot-v1.0 -- -c "cargo run --release --bin telegram_bot"

.PHONY: start
start:
	pm2 start polymarket-bot

.PHONY: start-telegram-bot
start-telegram-bot:
	pm2 start telegram-bot

.PHONY: stop
stop:
	pm2 stop polymarket-bot

.PHONY: stop-telegram-bot
stop-telegram-bot:
	pm2 stop telegram-bot

# -----------------------------------------------------------------------------
# Windows cross-build (optional)
# -----------------------------------------------------------------------------
.PHONY: build-x86_64
build-x86_64:
	@rustup target add $(TARGET_X86_64) 2>/dev/null || true
	$(CARGO) build --target=$(TARGET_X86_64) --release

.PHONY: build-i686
build-i686:
	@rustup target add $(TARGET_I686) 2>/dev/null || true
	$(CARGO) build --target=$(TARGET_I686) --release
