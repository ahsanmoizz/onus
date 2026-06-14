#!/usr/bin/env node
// Onus one-command installer for npm/npx users.
// Usage: npx @onus/install
// Downloads and runs the appropriate platform install script.

const { execSync } = require('child_process');
const https = require('https');
const fs = require('fs');
const os = require('os');
const path = require('path');

const VERSION = process.env.ONUS_VERSION || 'latest';
const REPO = 'Gitlawb/onus';

console.log('');
console.log('╔══════════════════════════════════════════════╗');
console.log('║        Onus — AI Agent Firewall             ║');
console.log('╚══════════════════════════════════════════════╝');
console.log('');

const platform = os.platform();

function download(url) {
    return new Promise((resolve, reject) => {
        https.get(url, (res) => {
            if (res.statusCode === 302 || res.statusCode === 301) {
                download(res.headers.location).then(resolve).catch(reject);
                return;
            }
            if (res.statusCode !== 200) {
                reject(new Error(`HTTP ${res.statusCode}`));
                return;
            }
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve(data));
        }).on('error', reject);
    });
}

async function main() {
    const isWin = platform === 'win32';
    const scriptName = isWin ? 'install.ps1' : 'install.sh';
    const ext = isWin ? 'ps1' : 'sh';

    // For "latest" tag, use raw from main branch.
    // For specific versions, use the tag.
    const ref = VERSION === 'latest' ? 'main' : VERSION;
    const scriptUrl = `https://raw.githubusercontent.com/${REPO}/${ref}/install/${scriptName}`;

    console.log(`  Downloading installer for ${platform}...`);
    console.log(`  ${scriptUrl}`);

    try {
        const script = await download(scriptUrl);
        const tmpFile = path.join(os.tmpdir(), `onus-install-${Date.now()}.${ext}`);
        fs.writeFileSync(tmpFile, script, 'utf-8');

        if (isWin) {
            console.log('  Running PowerShell installer...');
            execSync(`powershell -ExecutionPolicy Bypass -File "${tmpFile}"`, { stdio: 'inherit' });
        } else {
            console.log('  Running installer...');
            execSync(`bash "${tmpFile}"`, { stdio: 'inherit' });
        }

        fs.unlinkSync(tmpFile);
        console.log('');
        console.log('  Done. Restart your terminal or run: source ~/.bashrc');
    } catch (err) {
        console.error(`  Installation failed: ${err.message}`);
        console.log('');
        console.log('  Manual install:');
        console.log(`    curl -fsSL https://github.com/${REPO}/releases/latest/download/install.sh | bash`);
        process.exit(1);
    }
}

main();
