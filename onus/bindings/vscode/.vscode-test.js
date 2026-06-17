const { defineConfig } = require('@vscode/test-cli');

module.exports = defineConfig({
    label: 'Onus Extension Tests',
    files: 'test/**/*.test.js',
    version: '1.124.2',
    workspaceFolder: 'test/workspace',
    mocha: {
        ui: 'tdd',
        timeout: 10000,
    },
});
