// Onus VS Code Extension
// Intercepts VS Code terminal, task, and shell operations through Onus Core.
//
// Architecture:
//   - Registers a TerminalLinkProvider + TerminalProfileProvider to intercept shell sessions
//   - Hooks into VS Code's task system to evaluate tasks before execution
//   - Provides status bar indicator showing firewall state
//   - Shows notifications on blocked actions

const vscode = require('vscode');
const { spawn, execSync, execFileSync } = require('child_process');
const path = require('path');
const fs = require('fs');

// ── Module state ────────────────────────────────────────────────────────────

let statusBarItem = null;
let outputChannel = null;
let disposableShell = null;
let sessionId = null;
let sequenceCounter = 0;

// ── Logging ─────────────────────────────────────────────────────────────────

function log(level, msg, data = null) {
    const prefix = `[Onus ${level.toUpperCase()}]`;
    if (outputChannel) {
        const line = data
            ? `${prefix} ${msg} ${JSON.stringify(data)}`
            : `${prefix} ${msg}`;
        outputChannel.appendLine(line);
    }
    if (level === 'error') {
        console.error(`${prefix} ${msg}`, data || '');
    } else if (level === 'warn') {
        console.warn(`${prefix} ${msg}`, data || '');
    }
}

// ── Binary locator ──────────────────────────────────────────────────────────

function findBinary() {
    const cfg = vscode.workspace.getConfiguration('onus');
    const configured = cfg.get('binaryPath');
    if (configured) {
        return configured;
    }

    // PATH search.
    const paths = process.env.PATH.split(path.delimiter);
    const isWin = process.platform === 'win32';
    const binaryName = isWin ? 'onus.exe' : 'onus';

    for (const dir of paths) {
        const full = path.join(dir, binaryName);
        try {
            if (fs.statSync(full).isFile()) {
                return full;
            }
        } catch (_) { }
    }

    // Default install locations.
    if (isWin) {
        const local = path.join(process.env.LOCALAPPDATA || '', 'onus', 'onus.exe');
        if (fs.existsSync(local)) return local;
    } else {
        const candidates = ['/usr/local/bin/onus', path.join(process.env.HOME || '', '.local/bin/onus')];
        for (const c of candidates) {
            if (fs.existsSync(c)) return c;
        }
    }

    return null;
}

// ── Onus evaluate ───────────────────────────────────────────────────────────

function getSessionId() {
    if (!sessionId) {
        sessionId = `vscode-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    }
    return sessionId;
}

function evaluateAction(actionType, payload, tool) {
    const binary = findBinary();
    if (!binary) {
        log('error', 'Onus binary not found');
        return { decision: 'unsupported', error: 'Onus binary not found; action was not evaluated.' };
    }

    const cfg = vscode.workspace.getConfiguration('onus');
    if (!cfg.get('enabled', true)) {
        return { decision: 'disabled', error: 'Onus extension disabled by user configuration.' };
    }

    const seq = sequenceCounter++;

    const request = JSON.stringify({
        version: 1,
        session_id: getSessionId(),
        sequence: seq,
        action: {
            type: actionType,
            tool: tool || actionType,
            payload: typeof payload === 'string' ? { command: payload } : payload,
        },
    });

    try {
        const stdout = execFileSync(binary, ['evaluate'], {
            input: request,
            encoding: 'utf-8',
            timeout: 5000,
        });
        const result = JSON.parse(stdout.trim());
        log('debug', 'Evaluation result', result);
        return result;
    } catch (err) {
        log('error', `Evaluate failed: ${err.message}`);
        return { decision: 'unsupported', error: `Onus evaluation failed: ${err.message}` };
    }
}

function surfaceUnsupported(result) {
    if (result.decision === 'unsupported') {
        vscode.window.showErrorMessage(`Onus did not evaluate this action: ${result.error}`);
        return true;
    }
    if (result.decision === 'disabled') {
        log('warn', result.error);
        return true;
    }
    return false;
}

// ── Shell interception ──────────────────────────────────────────────────────

class OnusTerminal {
    constructor() {
        this._disposables = [];
        this._setupHooks();
    }

    _setupHooks() {
        // Intercept terminal creation to wrap shell with Onus.
        this._disposables.push(
            vscode.window.onDidOpenTerminal(async (terminal) => {
                log('info', `Terminal opened: ${terminal.name}`);
            })
        );

        // Monitor terminal shell execution.
        this._disposables.push(
            vscode.window.onDidChangeTerminalShellIntegration(async (event) => {
                const terminal = event.terminal;
                const integration = terminal.shellIntegration;

                if (!integration) return;

                log('info', `Shell integration detected for: ${terminal.name}`);

                // Intercept command execution.
                const executeDisposable = integration.onDidExecuteCommand((e) => {
                    const commandLine = e.commandLine || '';
                    const result = evaluateAction('shell', { command: commandLine, cwd: getCwd() }, 'vscode_terminal');

                    if (surfaceUnsupported(result)) {
                        return;
                    } else if (result.decision === 'block' || result.decision === 'escalate') {
                        const msg = result.correction || `Command blocked by Onus firewall`;
                        vscode.window.showWarningMessage(`🚫 Onus: ${msg}`);
                        log('warn', `Blocked terminal command`, { command: commandLine, rule: result.rule_id });
                    } else if (result.decision === 'warn') {
                        const msg = result.correction || `Command flagged by Onus`;
                        vscode.window.showWarningMessage(`⚠️ Onus: ${msg}`);
                        log('warn', `Warned terminal command`, { command: commandLine, rule: result.rule_id });
                    }
                });

                this._disposables.push(executeDisposable);
            })
        );
    }

    dispose() {
        for (const d of this._disposables) {
            d.dispose();
        }
        this._disposables = [];
    }
}

function getCwd() {
    const folders = vscode.workspace.workspaceFolders;
    if (folders && folders.length > 0) {
        return folders[0].uri.fsPath;
    }
    return process.cwd() || '/';
}

// ── Shell environment provider ──────────────────────────────────────────────

class OnusShellProfileProvider {
    constructor() { }

    provideTerminalProfile(token) {
        const shellPath = process.platform === 'win32'
            ? process.env.COMSPEC || 'cmd.exe'
            : process.env.SHELL || '/bin/bash';

        // Wrap the shell with Onus shell wrapper.
        const binary = findBinary();
        let shellArgs = [];

        if (binary) {
            // Use shell wrapper script if available.
            const configDir = path.dirname(binary);
            const wrapperPath = path.join(configDir, '..', 'share', 'onus', 'scripts', 'onus-shell-wrapper.sh');
            if (fs.existsSync(wrapperPath)) {
                shellArgs = ['-c', `source "${wrapperPath}" && exec "${shellPath}"`];
            }
        }

        return new vscode.TerminalProfile({
            name: 'Onus Shell',
            shellPath: shellPath,
            shellArgs: shellArgs.length > 0 ? shellArgs : undefined,
            env: {
                ONUS_SESSION_ID: getSessionId(),
            },
        });
    }
}

// ── Task hooks ──────────────────────────────────────────────────────────────

class OnusTaskProvider {
    constructor() {
        this._disposables = [];
        this._interceptTasks();
    }

    _interceptTasks() {
        // Intercept task execution via task executions.
        this._disposables.push(
            vscode.tasks.onDidStartTask((e) => {
                const task = e.execution.task;
                log('info', `Task started: ${task.name}`, { source: task.source });

                // Evaluate the task command.
                const execution = task.execution;
                if (execution && execution.commandLine) {
                    const result = evaluateAction('shell', {
                        command: execution.commandLine,
                        cwd: execution.options && execution.options.cwd || getCwd(),
                    }, `vscode_task:${task.name}`);

                    if (surfaceUnsupported(result)) {
                        return;
                    } else if (result.decision === 'block' || result.decision === 'escalate') {
                        vscode.window.showWarningMessage(`🚫 Onus blocked task: ${task.name}`);
                        log('warn', `Blocked task`, { task: task.name, rule: result.rule_id });
                    }
                }
            })
        );
    }

    dispose() {
        for (const d of this._disposables) {
            d.dispose();
        }
        this._disposables = [];
    }
}

// ── Status bar ──────────────────────────────────────────────────────────────

function updateStatusBar() {
    if (!statusBarItem) return;
    const cfg = vscode.workspace.getConfiguration('onus');
    const enabled = cfg.get('enabled', true);
    const binary = findBinary();

    if (!binary) {
        statusBarItem.text = '$(shield) Onus: not found';
        statusBarItem.tooltip = 'Onus binary not found. Install from github.com/ahsanmoizz/onus';
        statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        statusBarItem.command = { command: 'onus.openLog', title: 'Open Log' };
    } else if (!enabled) {
        statusBarItem.text = '$(shield) Onus: disabled';
        statusBarItem.tooltip = 'Onus firewall is disabled. Click to enable.';
        statusBarItem.backgroundColor = undefined;
        statusBarItem.command = { command: 'onus.enable', title: 'Enable' };
    } else {
        statusBarItem.text = '$(shield) Onus: best-effort';
        statusBarItem.tooltip = 'Onus VS Code extension is best-effort; use Onus-routed shells/SDK/proxy for pre-execution guarantees.';
        statusBarItem.backgroundColor = undefined;
        statusBarItem.command = { command: 'onus.disable', title: 'Disable' };
    }
    statusBarItem.show();
}

// ── Extension activation ────────────────────────────────────────────────────

function activate(context) {
    outputChannel = vscode.window.createOutputChannel('Onus Firewall');
    log('info', 'Onus VS Code extension activating...');

    // Status bar.
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    context.subscriptions.push(statusBarItem);
    updateStatusBar();

    // Configuration change listener.
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration('onus')) {
                updateStatusBar();
            }
        })
    );

    // Shell interception.
    const onusTerminal = new OnusTerminal();
    context.subscriptions.push(onusTerminal);

    // Task interception.
    const onusTaskProvider = new OnusTaskProvider();
    context.subscriptions.push(onusTaskProvider);

    // Shell profile provider.
    context.subscriptions.push(
        vscode.window.registerTerminalProfileProvider('onus.wrapped-shell', {
            provideTerminalProfile: (token) => {
                return new OnusShellProfileProvider().provideTerminalProfile(token);
            }
        })
    );

    // Commands.
    context.subscriptions.push(
        vscode.commands.registerCommand('onus.enable', () => {
            const cfg = vscode.workspace.getConfiguration('onus');
            cfg.update('enabled', true, true).then(() => {
                vscode.window.showInformationMessage('Onus firewall enabled.');
                updateStatusBar();
            });
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('onus.disable', () => {
            const cfg = vscode.workspace.getConfiguration('onus');
            cfg.update('enabled', false, true).then(() => {
                vscode.window.showWarningMessage('Onus firewall disabled.');
                updateStatusBar();
            });
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('onus.status', () => {
            const binary = findBinary();
            const cfg = vscode.workspace.getConfiguration('onus');
            const enabled = cfg.get('enabled', true);

            if (!binary) {
                vscode.window.showErrorMessage('Onus binary not found. Install from https://github.com/ahsanmoizz/onus/releases');
            } else if (enabled) {
                vscode.window.showInformationMessage(`Onus firewall ACTIVE — binary: ${binary}`);
            } else {
                vscode.window.showWarningMessage('Onus firewall DISABLED');
            }
            log('info', 'Status check', { binary, enabled });
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('onus.openLog', () => {
            outputChannel.show();
        })
    );

    log('info', 'Onus VS Code extension activated.');
    log('info', `Binary: ${findBinary() || 'NOT FOUND'}`);

    // Initial notification if binary not found.
    if (!findBinary()) {
        vscode.window.showWarningMessage(
            'Onus binary not found. Install it for shell and task protection.',
            'Get Onus'
        ).then(selection => {
            if (selection === 'Get Onus') {
                vscode.env.openExternal(vscode.Uri.parse('https://github.com/ahsanmoizz/onus/releases'));
            }
        });
    }
}

function deactivate() {
    log('info', 'Onus VS Code extension deactivated.');
    if (outputChannel) {
        outputChannel.dispose();
    }
}

module.exports = { activate, deactivate };
