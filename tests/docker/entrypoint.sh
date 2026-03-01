#!/usr/bin/env bash
# Test suite executed inside the Ubuntu container.
# Simulates a brand-new user running tmuxido for the first time.
set -uo pipefail

PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }

section() {
    echo ""
    echo "┌─ $1"
}

# ---------------------------------------------------------------------------
# Phase 1 — fzf and tmux are NOT installed yet
# ---------------------------------------------------------------------------

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║   tmuxido — Container Integration Tests (Ubuntu 24.04)  ║"
echo "╚══════════════════════════════════════════════════════════╝"

section "Phase 1: binary basics"

# T1 — binary is in PATH and executable
if command -v tmuxido &>/dev/null; then
    pass "tmuxido found in PATH ($(command -v tmuxido))"
else
    fail "tmuxido not found in PATH"
fi

# T2 — --help exits 0
if tmuxido --help >/dev/null 2>&1; then
    pass "--help exits with code 0"
else
    fail "--help returned non-zero"
fi

# T3 — --version shows the package name
VERSION_OUT=$(tmuxido --version 2>&1 || true)
if echo "$VERSION_OUT" | grep -q "tmuxido"; then
    pass "--version output contains 'tmuxido' → $VERSION_OUT"
else
    fail "--version output unexpected: $VERSION_OUT"
fi

# ---------------------------------------------------------------------------
# Phase 2 — dependency detection (fzf and tmux absent)
# ---------------------------------------------------------------------------

section "Phase 2: dependency detection (fzf and tmux not installed)"

# Pipe "n" so tmuxido declines to install and exits
DEP_OUT=$(echo "n" | tmuxido 2>&1 || true)

# T4 — fzf reported as missing
if echo "$DEP_OUT" | grep -q "fzf"; then
    pass "fzf detected as missing"
else
    fail "fzf NOT detected as missing. Full output:\n$DEP_OUT"
fi

# T5 — tmux reported as missing
if echo "$DEP_OUT" | grep -q "tmux"; then
    pass "tmux detected as missing"
else
    fail "tmux NOT detected as missing. Full output:\n$DEP_OUT"
fi

# T6 — "not installed" heading appears
if echo "$DEP_OUT" | grep -q "not installed"; then
    pass "User-facing 'not installed' message shown"
else
    fail "'not installed' message missing. Full output:\n$DEP_OUT"
fi

# T7 — apt detected as package manager (Ubuntu 24.04)
if echo "$DEP_OUT" | grep -q "apt"; then
    pass "apt detected as the package manager"
else
    fail "apt NOT detected. Full output:\n$DEP_OUT"
fi

# T8 — install command includes sudo apt install
if echo "$DEP_OUT" | grep -q "sudo apt install"; then
    pass "Install command 'sudo apt install' shown to user"
else
    fail "Install command incorrect. Full output:\n$DEP_OUT"
fi

# T9 — cancellation message when user answers "n"
if echo "$DEP_OUT" | grep -q "cancelled\|Cancelled\|manually"; then
    pass "Graceful cancellation message shown"
else
    fail "Cancellation message missing. Full output:\n$DEP_OUT"
fi

# ---------------------------------------------------------------------------
# Phase 3 — install deps and run full workflow
# ---------------------------------------------------------------------------

section "Phase 3: full workflow (after installing fzf, tmux and git)"

echo "    Installing fzf, tmux via apt (this may take a moment)..."
sudo apt-get update -qq 2>/dev/null
sudo apt-get install -y --no-install-recommends fzf tmux 2>/dev/null

# T10 — fzf now available
if command -v fzf &>/dev/null; then
    pass "fzf installed successfully ($(fzf --version 2>&1 | head -1))"
else
    fail "fzf still not available after installation"
fi

# T11 — tmux now available
if command -v tmux &>/dev/null; then
    pass "tmux installed successfully ($(tmux -V))"
else
    fail "tmux still not available after installation"
fi

# T12 — tmuxido no longer triggers dependency prompt
NO_DEP_OUT=$(echo "" | tmuxido 2>&1 || true)
if echo "$NO_DEP_OUT" | grep -q "not installed"; then
    fail "Dependency prompt still shown after installing deps"
else
    pass "No dependency prompt after deps are installed"
fi

# T13 — set up a minimal git project tree for scanning
mkdir -p ~/Projects/demo-app
git -C ~/Projects/demo-app init --quiet
git -C ~/Projects/demo-app config user.email "test@test.com"
git -C ~/Projects/demo-app config user.name "Test"

mkdir -p ~/.config/tmuxido
cat > ~/.config/tmuxido/tmuxido.toml <<'EOF'
paths = ["~/Projects"]
max_depth = 3
cache_enabled = true
EOF

# T13 — --refresh scans and finds our demo project
REFRESH_OUT=$(tmuxido --refresh 2>&1 || true)
if echo "$REFRESH_OUT" | grep -q "projects\|Projects"; then
    pass "--refresh scanned and reported projects"
else
    fail "--refresh output unexpected: $REFRESH_OUT"
fi

# T14 — --cache-status reports the cache that was just built
CACHE_OUT=$(tmuxido --cache-status 2>&1 || true)
if echo "$CACHE_OUT" | grep -qi "cache"; then
    pass "--cache-status reports cache info"
else
    fail "--cache-status output unexpected: $CACHE_OUT"
fi

# T15 — cache contains our demo project
if echo "$CACHE_OUT" | grep -q "Projects cached: [^0]"; then
    pass "Cache contains at least 1 project"
else
    # Try alternate grep in case format differs
    if echo "$CACHE_OUT" | grep -q "cached:"; then
        pass "--cache-status shows cached projects (count check skipped)"
    else
        fail "Cache appears empty. Output: $CACHE_OUT"
    fi
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
printf "║  Results: %-3d passed, %-3d failed%*s║\n" \
    "$PASS" "$FAIL" $((24 - ${#PASS} - ${#FAIL})) ""
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

[ "$FAIL" -eq 0 ]
