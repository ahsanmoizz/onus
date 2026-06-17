const { defineConfig } = require('@vscode/test-cli');

module.exports = defineConfig({
    label: 'Onus Antigravity Extension Tests',
    files: 'test/**/*.test.js',
    version: '1.107.0',
    platform: 'win32-x64',
    workspaceFolder: 'test/workspace',
    mocha: {
        ui: 'tdd',
        timeout: 10000,
    },
});
