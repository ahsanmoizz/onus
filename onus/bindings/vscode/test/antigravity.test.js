//
// Onus Antigravity Extension Test Runner
//
// Tests the Onus extension running inside Google Antigravity.
// Uses the @vscode/test-cli runner with Antigravity binary.
//
// Run: npx @vscode/test-cli run --config .antigravity-test.js
//

const assert = require('assert');
const path = require('path');
const fs = require('fs');

suite('Onus Antigravity Extension', () => {
    test('Extension loads and activates', async () => {
        const ext = vscode.extensions.getExtension('onus.onus-firewall');
        assert.ok(ext, 'Extension onus.onus-firewall should be present');
        await ext.activate();
        assert.strictEqual(ext.isActive, true, 'Extension should activate');
    });

    test('Extension registers onus.doctor command', async () => {
        const commands = await vscode.commands.getCommands(true);
        const found = commands.some(cmd => cmd === 'onus.doctor');
        assert.ok(found, 'onus.doctor command should be registered');
    });

    test('Extension registers onus.setup command', async () => {
        const commands = await vscode.commands.getCommands(true);
        const found = commands.some(cmd => cmd === 'onus.setup');
        assert.ok(found, 'onus.setup command should be registered');
    });

    test('Extension registers onus.statusBarItem', async () => {
        // The extension creates a status bar item named 'onus.status'
        const commands = await vscode.commands.getCommands(true);
        const found = commands.some(cmd => cmd === 'onus.status');
        assert.ok(found, 'onus.status command should be registered');
    });

    test('Doctor command produces output', async () => {
        const result = await vscode.commands.executeCommand('onus.doctor');
        assert.ok(result !== undefined, 'Doctor command should return output');
    });

    test('Extension detects daemon status', async () => {
        const ext = vscode.extensions.getExtension('onus.onus-firewall');
        await ext.activate();
        const api = ext.exports;
        assert.ok(api, 'Extension should export an API');
        // The API should have a getStatus method
        assert.strictEqual(typeof api.getDaemonStatus, 'function',
            'Extension API should expose getDaemonStatus');
    });
});
