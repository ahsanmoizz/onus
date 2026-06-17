//
// Onus Antigravity Extension — Non-interactive verification runner
//
// Tests Antigravity integration without launching the GUI:
//   1. Binary presence and version
//   2. Extension deployment
//   3. MCP configuration
//   4. CLI commands (doctor, setup, uninstall)
//
// Usage: node test/antigravity-verify.js
//

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const ANTIGRAVITY_BIN = 'D:\\Antigravity\\bin\\antigravity';
const ONUS_BIN = path.resolve(__dirname, '..', '..', '..', 'onus',
    'target', 'debug', 'onus.exe');
const EXTENSIONS_DIR = path.join(
    process.env.USERPROFILE || 'C:\\Users\\A',
    '.antigravity', 'extensions', 'onus.onus-firewall-0.1.0'
);

let passed = 0;
let failed = 0;
const results = [];

function test(name, fn) {
    try {
        fn();
        results.push({ name, status: 'PASS' });
        passed++;
    } catch (e) {
        results.push({ name, status: 'FAIL', error: e.message });
        failed++;
    }
}

function run(cmd, opts = {}) {
    return execSync(cmd, {
        encoding: 'utf-8',
        timeout: 15000,
        ...opts,
    });
}

// ── Antigravity Binary Tests ─────────────────────────────────────────

test('Antigravity binary exists', () => {
    assert(fs.existsSync(ANTIGRAVITY_BIN),
        `Antigravity binary not found at ${ANTIGRAVITY_BIN}`);
});

test('Antigravity --version succeeds', () => {
    const out = run(`"${ANTIGRAVITY_BIN}" --version`);
    assert(out.includes('1.107.0'), `Version mismatch: ${out.trim()}`);
});

test('Antigravity --list-extensions contains onus-firewall', () => {
    const out = run(`"${ANTIGRAVITY_BIN}" --list-extensions`);
    assert(out.includes('onus.onus-firewall'),
        `Extension not listed: ${out}`);
});

// ── Extension Deployment Tests ───────────────────────────────────────

test('Extension directory exists', () => {
    assert(fs.existsSync(EXTENSIONS_DIR),
        `Extension dir not found: ${EXTENSIONS_DIR}`);
});

test('Extension package.json exists', () => {
    const pkgPath = path.join(EXTENSIONS_DIR, 'package.json');
    assert(fs.existsSync(pkgPath), 'package.json not found');
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
    assert.strictEqual(pkg.name, 'onus-firewall', 'Wrong extension name');
    assert(pkg.version, 'Missing version');
});

test('Extension main entry exists', () => {
    const pkg = JSON.parse(
        fs.readFileSync(path.join(EXTENSIONS_DIR, 'package.json'), 'utf-8')
    );
    const mainPath = path.join(EXTENSIONS_DIR, pkg.main);
    assert(fs.existsSync(mainPath), `Main entry not found: ${mainPath}`);
});

test('Extension contributes commands', () => {
    const pkg = JSON.parse(
        fs.readFileSync(path.join(EXTENSIONS_DIR, 'package.json'), 'utf-8')
    );
    assert(pkg.contributes, 'Missing contributes');
    assert(Array.isArray(pkg.contributes.commands),
        'Missing contributes.commands array');
    assert(pkg.contributes.commands.length > 0,
        'No contributed commands');
});

// ── Onus CLI Antigravity Tests ───────────────────────────────────────

test('onus doctor --antigravity succeeds', () => {
    assert(fs.existsSync(ONUS_BIN), `Onus binary not found: ${ONUS_BIN}`);
    const out = run(`"${ONUS_BIN}" doctor --antigravity`);
    assert(out.includes('Antigravity'), 'Output missing Antigravity header');
    assert(out.includes('onus-firewall'), 'Output missing extension info');
});

test('onus doctor (full) includes Antigravity section', () => {
    const out = run(`"${ONUS_BIN}" doctor`);
    assert(out.includes('Antigravity') || out.includes('antigravity'),
        'Full doctor output missing Antigravity');
});

test('onus setup --antigravity succeeds', () => {
    const out = run(`"${ONUS_BIN}" setup --antigravity`);
    assert(out.includes('Antigravity'), 'Output missing Antigravity header');
});

test('onus uninstall --antigravity succeeds', () => {
    const out = run(`"${ONUS_BIN}" uninstall --antigravity`);
    assert(out.includes('Antigravity') || out.includes('antigravity'),
        'Output missing Antigravity reference');
});

// ── Report ───────────────────────────────────────────────────────────

function assert(condition, message) {
    if (!condition) throw new Error(message);
}

console.log('\n=== Onus Antigravity Verification ===\n');
results.forEach(r => {
    const sym = r.status === 'PASS' ? '✓' : '✗';
    console.log(`  ${sym} ${r.name}`);
    if (r.status === 'FAIL') {
        console.log(`      ${r.error}`);
    }
});
console.log(`\n  ${passed} passed, ${failed} failed, ${passed + failed} total\n`);

if (failed > 0) {
    process.exit(1);
}
