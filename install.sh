#!/usr/bin/env bash
# Knowledge Loom installer
#
# Usage (run from your knowledge directory):
#   curl -fsSL https://raw.githubusercontent.com/odinkirk/knowledge-loom/main/install.sh | bash
#
# To install to a custom directory (default: .loom):
#   LOOM_DIR=.knowledge-loom curl -fsSL ... | bash
#
# The script is idempotent — re-running it updates the tool files and preserves
# any existing MCP servers in .mcp.json.

set -euo pipefail

LOOM_DIR="${LOOM_DIR:-.loom}"
REPO_BASE="https://raw.githubusercontent.com/odinkirk/knowledge-loom/main"
KB_ROOT="$(pwd)"

echo "Installing Knowledge Loom to $KB_ROOT/$LOOM_DIR ..."
echo ""

# ── 1. Create install directory ───────────────────────────────────────────────
mkdir -p "$LOOM_DIR"

# ── 2. Download tool files ────────────────────────────────────────────────────
echo "Downloading tool files..."
for f in kb_core.py loom_mcp.py requirements.txt; do
    curl -fsSL "$REPO_BASE/$f" -o "$LOOM_DIR/$f"
done
echo "  kb_core.py, loom_mcp.py, requirements.txt"

# ── 3. Set up Python environment ──────────────────────────────────────────────
# Priority: uv (handles PEP 668 managed Python) → python3 -m venv → error + guidance
echo ""
echo "Setting up Python environment..."

if command -v uv &>/dev/null; then
    echo "  Using uv..."
    uv venv "$LOOM_DIR/.venv" --quiet
    uv pip install -r "$LOOM_DIR/requirements.txt" --python "$LOOM_DIR/.venv" --quiet
elif python3 -m venv "$LOOM_DIR/.venv" 2>/dev/null; then
    echo "  Using python3 -m venv..."
    "$LOOM_DIR/.venv/bin/pip" install -r "$LOOM_DIR/requirements.txt" -q
else
    echo ""
    echo "ERROR: Cannot create a virtual environment."
    echo ""
    echo "Your Python installation appears to be externally managed (PEP 668)."
    echo "Install uv to resolve this:"
    echo ""
    echo "  macOS/Linux:  curl -LsSf https://astral.sh/uv/install.sh | sh"
    echo "  Homebrew:     brew install uv"
    echo "  pip:          pip install uv"
    echo ""
    echo "Then re-run this installer."
    exit 1
fi

PYTHON="$LOOM_DIR/.venv/bin/python3"

# ── 4. Merge loom server entry into .mcp.json ─────────────────────────────────
# Only adds/updates the "loom" key — all other MCP servers are preserved exactly.
# Backs up .mcp.json.bak if the existing file contains invalid JSON.
echo ""
echo "Configuring .mcp.json..."

"$PYTHON" - <<PYEOF
import json, os, shutil

cfg_path = ".mcp.json"
cfg = {"mcpServers": {}}

if os.path.exists(cfg_path):
    try:
        with open(cfg_path) as f:
            cfg = json.load(f)
        if "mcpServers" not in cfg:
            cfg["mcpServers"] = {}
    except json.JSONDecodeError:
        backup = cfg_path + ".bak"
        shutil.copy(cfg_path, backup)
        print(f"  WARNING: {cfg_path} contained invalid JSON — backed up to {backup}")
        cfg = {"mcpServers": {}}

cfg["mcpServers"]["loom"] = {
    "command": os.path.abspath("$LOOM_DIR/.venv/bin/python3"),
    "args": [os.path.abspath("$LOOM_DIR/loom_mcp.py")],
    "env": {"KB_ROOT": "$KB_ROOT"},
}

with open(cfg_path, "w") as f:
    json.dump(cfg, f, indent=2)
    f.write("\n")

other = [k for k in cfg["mcpServers"] if k != "loom"]
if other:
    print(f"  Preserved existing servers: {', '.join(other)}")
print(f"  Added 'loom' to {cfg_path}")
PYEOF

# ── 5. Add install dir to .gitignore (if inside a git repo) ──────────────────
if [ -d .git ]; then
    if ! grep -qF "$LOOM_DIR" .gitignore 2>/dev/null; then
        printf "\n# Knowledge Loom (managed by install.sh)\n%s/\n" "$LOOM_DIR" >> .gitignore
        echo ""
        echo "Added $LOOM_DIR/ to .gitignore"
    fi
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "Knowledge Loom installed successfully."
echo ""
echo "  Location : $KB_ROOT/$LOOM_DIR/"
echo "  KB root  : $KB_ROOT"
echo ""
echo "Restart Claude Code and run /mcp to verify the 'loom' server is connected."
echo ""
echo "Optional — enable additional backends by adding env vars to .mcp.json:"
echo "  VAULT_PATH    — path to Obsidian vault (enables graph analytics)"
echo "  BRAINJAR_PATH — path to brainjar binary (enables loom_search_smart)"
