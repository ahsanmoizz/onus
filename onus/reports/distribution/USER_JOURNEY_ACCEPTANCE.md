# Onus User Journey Acceptance Report

**Version:** 0.1.0
**Date:** 2026-06-19

---

## User Personas

| Persona | Description | Primary Path |
|---------|-------------|-------------|
| **Individual Developer** | Solo dev wanting to govern own AI coding agent | Install → Setup → Activate → Evaluate |
| **Team Lead** | Lead wanting to enforce policies across team | Install → Policy sign → Distribute → Audit |
| **Security Engineer** | Security professional evaluating Onus | Review → Test → Audit verify |
| **Enterprise Admin** | Deploying Onus org-wide | Scripted install → Managed policies → L3/L4 |

---

## Primary Journey: Individual Developer

### Step 1: Discover

**Entry points:**
- GitHub repository (github.com/ahsanmoizz/onus)
- Website landing page (onus.ai)
- Social media / word of mouth

**Content consumed:**
- `/` — Landing page with problem/solution
- `/how-it-works` — Governance pipeline explanation
- `/security` — Security model
- `/guardian-modes` — Mode comparison

**Expected:** User understands what Onus does and why they need it.

### Step 2: Install

**Entry points:**
- `/install` — Install instructions
- `/download` — Direct download

**Artifacts consumed:**
- Installer script (install-onus.ps1 or install-onus.sh)
- Release binary

**Expected:** Binary installed, `onus --help` works.

### Step 3: Configure

**Commands executed:**
```bash
onus doctor              # Verify everything is ready
onus daemon start        # Start the daemon
onus setup               # Interactive setup wizard
```

**Artifacts consumed:**
- `/docs/quick-start` — Quick start guide
- `/docs/providers` — Provider configuration
- `/docs/guardian-modes` — Mode selection

**Expected:** Provider configured, guardian mode selected, daemon running.

### Step 4: Activate

**Actions:**
- Open `onus console` or navigate to `http://localhost:9090`
- Run activation wizard at `/activate`
- Configure agent integration

**Commands executed:**
```bash
onus setup --claude-code    # or --codex, --cursor
```

**Expected:** Agent integration active, console shows healthy status.

### Step 5: Govern

**Commands executed:**
```bash
onus evaluate --prompt "your prompt"   # Test evaluation
onus session start                      # Start a governed session
```

**Artifacts consumed:**
- `/docs/rules-policies` — Understanding rules
- `/docs/running-governed-tasks` — Running tasks
- Console dashboard — Real-time activity feed

**Expected:** Actions evaluated, audit trail populated, dashboard shows activity.

### Step 6: Review

**Console pages visited:**
- `/` — Dashboard with stats and activity
- `/actions` — Action history
- `/audit` — Audit trail
- `/sessions` — Session management

**Expected:** User can review all governed activity.

### Step 7: Approve

**If using Professional or Enterprise Strict mode:**
- `/approvals` — Pending approvals list
- Approval requires real-time binding

**Expected:** Approval flow works for actions needing human review.

---

## Journey Quality Assessment

| Stage | Experience | Gaps |
|-------|-----------|------|
| **Discover** | Full marketing site with all documentation | No video demo; no interactive playground |
| **Install** | Installers with SHA-256 verification | No Homebrew/choco/scoop package |
| **Configure** | Interactive setup wizard, doctor command | No first-run guided tour |
| **Activate** | Wizard UI, CLI integration commands | Activation wizard uses simulated validation (VIOLATION) |
| **Govern** | Full evaluation pipeline, session management | Beginner mode may be too permissive for new users to see value |
| **Review** | 18 console pages with real API data | Some pages don't handle all error states gracefully |
| **Approve** | Real approval workflow with binding | Requires Professional mode or higher |

## Verdict

The user journey is functional from end to end. A developer can:
1. Find Onus → 2. Install → 3. Configure → 4. Govern their agent → 5. Review activity

Critical gap: The activation wizard uses simulated validation that must be replaced
with real daemon API calls before production release.

The clean-machine journey has not been tested (see CLEAN_MACHINE_ACCEPTANCE.md).
