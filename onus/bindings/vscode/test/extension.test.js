// Onus VS Code Extension — activation test suite
// Uses globals provided by VS Code extension host test runner.
// Checks are conditional — we handle the case where globals are absent.

if (typeof suite === 'undefined') {
    console.log('suite/test globals not available — extension loaded in host');
    return;
}

const vscode = require('vscode');
const assert = require('assert');

suite('Onus Firewall Extension', () => {

    test('Extension should be present and activate', async () => {
        const ext = vscode.extensions.getExtension('onus.onus-firewall');
        assert.ok(ext, 'Extension onus.onus-firewall should be present');
    });

    test('Extension should activate without error', async () => {
        const ext = vscode.extensions.getExtension('onus.onus-firewall');
        await ext.activate();
        assert.ok(ext.isActive, 'Extension should be active');
    });

    test('Extension registers onus commands', async () => {
        const commands = await vscode.commands.getCommands(true);
        const onusCommands = commands.filter(c => c.startsWith('onus.'));
        assert.ok(onusCommands.includes('onus.enable'), 'onus.enable should be registered');
        assert.ok(onusCommands.includes('onus.disable'), 'onus.disable should be registered');
        assert.ok(onusCommands.includes('onus.status'), 'onus.status should be registered');
        assert.ok(onusCommands.includes('onus.openLog'), 'onus.openLog should be registered');
    });

    test('Extension configuration schema is valid', async () => {
        const config = vscode.workspace.getConfiguration('onus');
        assert.ok(config.has('enabled'), 'onus.enabled config should exist');
        assert.ok(config.has('binaryPath'), 'onus.binaryPath config should exist');
        assert.ok(config.has('blockOnEscalate'), 'onus.blockOnEscalate config should exist');
        assert.ok(config.has('logLevel'), 'onus.logLevel config should exist');
    });

    test('Extension defaults to enabled', async () => {
        const config = vscode.workspace.getConfiguration('onus');
        assert.strictEqual(config.get('enabled'), true, 'onus.enabled should default to true');
    });

});
