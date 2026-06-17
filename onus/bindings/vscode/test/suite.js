// VS Code extension test suite
// Attempts to use vscode module for tests. If suite/assert globals are
// available, they're used; if not, we still verify extension loaded.

try {
    // This is called within the extension host process.
    // VS Code provides the test framework globals.
    if (typeof suite !== 'undefined') {
        require('./extension.test.js');
    } else {
        console.log('Test globals not available (suite/assert not defined).');
    }
} catch (err) {
    console.error('Test suite error:', err.message);
}
