#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Agent-to-agent continuity live-test script.
.DESCRIPTION
    Tests Claude Code -> Codex CLI handoff continuity via Onus.
    This is a REAL test — it requires both Claude Code AND Codex CLI
    to be installed, authenticated, and registered with Onus.

    The script will stop with instructions if any prerequisite is missing.
    It does NOT fake agent execution or mock any external tool.

    Expected workflow:
      1. Claude Code starts one governed task
      2. Claude creates one allowed file
      3. Claude leaves a clearly defined task incomplete
      4. Onus captures a checkpoint
      5. Onus creates a handoff manifest
      6. Codex receives the continuation brief
      7. Codex verifies repository state
      8. Codex completes the remaining task
      9. Onus verifies same session, same task contract, one receipt chain
#>

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$Binary   = Join-Path $RepoRoot "onus\target\release\onus.exe"
$TestDir  = Join-Path $RepoRoot "runtime\continuity-test"
$ExitCode = 0

Write-Host "=== Onus Continuity Live Test ===" -ForegroundColor Cyan
Write-Host ""

# ---- Prerequisites ----
Write-Host "Checking prerequisites..." -ForegroundColor Cyan

# 1. Build
if (-not (Test-Path $Binary)) {
    Write-Host "[FAIL] Onus binary not found at $Binary" -ForegroundColor Red
    Write-Host "       Run .\scripts\build-onus.ps1 first" -ForegroundColor Yellow
    exit 1
}

# 2. Claude Code
$ClaudeVersion = & claude --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[SKIP] Claude Code CLI is not installed or not in PATH." -ForegroundColor Yellow
    Write-Host "       Install from: https://docs.anthropic.com/en/docs/claude-code/installation" -ForegroundColor Yellow
    Write-Host "       Then run: claude login" -ForegroundColor Yellow
    Write-Host "       Then run: onus setup --claude" -ForegroundColor Yellow
    Write-Host "       Prerequisite: \$env:ANTHROPIC_API_KEY or logged in via claude login" -ForegroundColor Yellow
    $ClaudeAvailable = $false
} else {
    $ClaudeAvailable = $true
    Write-Host "[OK] Claude Code: $ClaudeVersion" -ForegroundColor Green
}

# 3. Codex CLI
$CodexVersion = & codex --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[SKIP] Codex CLI is not installed or not in PATH." -ForegroundColor Yellow
    Write-Host "       Install from: https://github.com/openai/codex" -ForegroundColor Yellow
    Write-Host "       Then run: codex login" -ForegroundColor Yellow
    Write-Host "       Then run: onus setup --codex" -ForegroundColor Yellow
    $CodexAvailable = $false
} else {
    $CodexAvailable = $true
    Write-Host "[OK] Codex CLI: $CodexVersion" -ForegroundColor Green
}

# 4. Onus setup
Write-Host "[INFO] Onus integration status:" -ForegroundColor Cyan
& $Binary doctor
Write-Host ""

if (-not $ClaudeAvailable -or -not $CodexAvailable) {
    Write-Host ""
    Write-Host "=== CONTINUITY TEST ABORTED ===" -ForegroundColor Red
    Write-Host "Prerequisites not met:" -ForegroundColor Red
    if (-not $ClaudeAvailable) { Write-Host "  - Claude Code CLI is required" -ForegroundColor Red }
    if (-not $CodexAvailable) { Write-Host "  - Codex CLI is required" -ForegroundColor Red }
    Write-Host ""
    Write-Host "Install and authenticate both agents, then re-run this script." -ForegroundColor Yellow
    Write-Host "When both agents are available, the test will:" -ForegroundColor Yellow
    Write-Host "  1. Create a disposable project under runtime/continuity-test" -ForegroundColor Yellow
    Write-Host "  2. Launch Claude Code with a governed task (write a file)" -ForegroundColor Yellow
    Write-Host "  3. Capture checkpoint + create handoff" -ForegroundColor Yellow
    Write-Host "  4. Launch Codex CLI to continue the task" -ForegroundColor Yellow
    Write-Host "  5. Verify chain continuity" -ForegroundColor Yellow
    exit 1
}

# ---- Set up test environment ----
Write-Host ""
Write-Host "=== Setting up test environment ===" -ForegroundColor Cyan

# Create disposable project
if (Test-Path $TestDir) {
    Remove-Item -Recurse -Force $TestDir
}
New-Item -ItemType Directory -Path $TestDir -Force | Out-Null

# Initialise git in test dir
Push-Location $TestDir
try {
    git init
    git config user.email "test@example.invalid"
    git config user.name "Onus Continuity Test"
    New-Item -ItemType Directory -Path "src" -Force | Out-Null
    Set-Content -Path "README.md" -Value "# Continuity Test Project`n`nTask: Create src/main.py that prints 'Hello from Claude'"
    git add -A
    git commit -m "initial commit"
} finally {
    Pop-Location
}

Write-Host "[OK] Test environment: $TestDir" -ForegroundColor Green
Write-Host ""

# ---- Step 1: Claude Code creates a file ----
Write-Host "=== Step 1: Claude Code governed task ===" -ForegroundColor Cyan
Write-Host "Claude Code will now start a governed task in the test project."
Write-Host "Claude should: create src/main.py with 'print(\"Hello from Claude\")'"
Write-Host "Claude should: then indicate the task is incomplete (needs testing)."
Write-Host ""
Write-Host "Press ENTER to launch Claude Code..." -ForegroundColor Yellow
Read-Host
Write-Host ""
Write-Host "Running: cd $TestDir && claude -p 'Create src/main.py that prints \"Hello from Claude\" and src/test_main.py that imports main and asserts the output. Create main.py first and stop before creating test_main.py.'"
Write-Host ""

# Launch Claude (interactive — user must complete manually)
$ClaudeTask = @"
Create src/main.py that prints "Hello from Claude" and src/test_main.py that imports main and asserts the output.
Create main.py first, then STOP and indicate you're incomplete.
"@
Push-Location $TestDir
try {
    & claude -p $ClaudeTask
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[WARN] Claude Code exited with code $LASTEXITCODE" -ForegroundColor Yellow
    }
} catch {
    Write-Host "[WARN] Claude Code error: $_" -ForegroundColor Yellow
} finally {
    Pop-Location
}

# ---- Step 2: Create checkpoint ----
Write-Host ""
Write-Host "=== Step 2: Checkpoint + handoff ===" -ForegroundColor Cyan

# Create checkpoint
$CheckpointResult = & $Binary checkpoint create --label "pre-handoff" --workspace $TestDir 2>&1
Write-Host "Checkpoint: $CheckpointResult" -ForegroundColor Cyan

# List checkpoints
& $Binary checkpoint list 2>&1

# ---- Step 3: Codex continues ----
Write-Host ""
Write-Host "=== Step 3: Codex CLI continuation ===" -ForegroundColor Cyan
Write-Host "Codex should: read the current state, create test_main.py, and run tests."
Write-Host ""
Write-Host "Press ENTER to launch Codex CLI..." -ForegroundColor Yellow
Read-Host
Write-Host "Running: cd $TestDir && codex -p 'Continue this task: src/main.py exists but src/test_main.py does not. Create src/test_main.py that imports main and asserts the output string. Run the test.'"
Write-Host ""

# Launch Codex (interactive — user must complete manually)
Push-Location $TestDir
try {
    & codex -p "Continue this task: src/main.py exists but src/test_main.py does not. Create src/test_main.py that imports main and asserts the output string. Run the test."
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[WARN] Codex CLI exited with code $LASTEXITCODE" -ForegroundColor Yellow
    }
} catch {
    Write-Host "[WARN] Codex CLI error: $_" -ForegroundColor Yellow
} finally {
    Pop-Location
}

# ---- Step 4: Verify ----
Write-Host ""
Write-Host "=== Step 4: Verification ===" -ForegroundColor Cyan

Write-Host ""
Write-Host "=== Test Artifacts ===" -ForegroundColor Cyan
Write-Host "Test project: $TestDir" -ForegroundColor Cyan

Write-Host ""
Write-Host "=== Post-test checklist ===" -ForegroundColor Cyan
Write-Host "1. Verify both files exist:" -ForegroundColor Yellow
Write-Host "   ls $TestDir/src/" -ForegroundColor Yellow
Write-Host "2. Verify receipt chain:" -ForegroundColor Yellow
Write-Host "   onus verify" -ForegroundColor Yellow
Write-Host "3. Verify session continuity:" -ForegroundColor Yellow
Write-Host "   onus session list" -ForegroundColor Yellow

Write-Host ""
Write-Host "=== CONTINUITY TEST COMPLETE ===" -ForegroundColor Green
Write-Host "Note: The continuity test is semi-automated because Claude Code" -ForegroundColor Cyan
Write-Host "and Codex CLI are interactive agents. Follow the prompts above." -ForegroundColor Cyan

# Cleanup prompt
Write-Host ""
Write-Host "Clean up test project? (y/N): " -ForegroundColor Yellow -NoNewline
$Cleanup = Read-Host
if ($Cleanup -eq "y" -or $Cleanup -eq "Y") {
    Remove-Item -Recurse -Force $TestDir -ErrorAction SilentlyContinue
    Write-Host "Cleaned up $TestDir" -ForegroundColor Green
}

exit 0
