# onus-shell-wrapper.sh
# Source this in your .bashrc / .zshrc to intercept every command through Onus.
#
# Usage:
#   source /path/to/onus-shell-wrapper.sh
#
# For single-session tracking:
#   source onus-shell-wrapper.sh && onus_shell_start "my task description"
#
# This uses the DEBUG trap + PROMPT_COMMAND to intercept commands
# before bash/zsh executes them.
#
# Environment variables:
#   ONUS_SHELL_ENABLED=1           (set to disable without removing source)
#   ONUS_BIN=/path/to/onus         (default: onus on PATH)
#   ONUS_SESSION_ID=<uuid>         (auto-generated if empty)

if [ -z "$ONUS_SHELL_ENABLED" ]; then
    ONUS_SHELL_ENABLED=1
fi

if [ -z "$ONUS_SESSION_ID" ]; then
    if command -v uuidgen > /dev/null 2>&1; then
        ONUS_SESSION_ID="shell-$(uuidgen)"
    else
        ONUS_SESSION_ID="shell-$(date +%s)-$$"
    fi
    export ONUS_SESSION_ID
fi

onus_bin() {
    echo "${ONUS_BIN:-onus}"
}

# Start an Onus session for shell tracking.
onus_shell_start() {
    local task="${1:-Interactive shell session}"
    export ONUS_SESSION_ID="shell-$(date +%s)-$$"
    ONUS_SEQUENCE=0
    # Mark session in audit trail (soft fail if daemon not running).
    echo "{\"version\":1,\"session_id\":\"$ONUS_SESSION_ID\",\"sequence\":0,\"action\":{\"type\":\"shell\",\"tool\":\"session_start\",\"payload\":{\"task\":\"$task\"}}}" | "$(onus_bin)" evaluate 2>/dev/null || true
    echo "Onus: session $ONUS_SESSION_ID started — \"$task\""
}

# End the Onus session.
onus_shell_end() {
    local session_id="${ONUS_SESSION_ID:-unknown}"
    echo "{\"version\":1,\"session_id\":\"$session_id\",\"sequence\":0,\"action\":{\"type\":\"shell\",\"tool\":\"session_end\",\"payload\":{}}}" | "$(onus_bin)" evaluate 2>/dev/null || true
    echo "Onus: session ended"
}

# Core function: evaluate a command through Onus.
# Returns 0 (allow), 1 (warn), 2+ (block).
onus_eval() {
    local cmd="$1"
    [ -z "$cmd" ] && return 0
    [ -z "$ONUS_SHELL_ENABLED" ] && return 0

    local seq="${ONUS_SEQUENCE:-0}"
    ONUS_SEQUENCE=$((seq + 1))

    # Escape the command for JSON.
    local escaped
    escaped=$(printf '%s' "$cmd" | sed 's/"/\\"/g' | sed ':a;N;$!ba;s/\n/\\n/g')

    # Build the Onus action request JSON.
    local json
    json="{\"version\":1,\"session_id\":\"$ONUS_SESSION_ID\",\"sequence\":$seq,\"action\":{\"type\":\"shell\",\"tool\":\"interactive_shell\",\"payload\":{\"command\":\"$escaped\",\"cwd\":\"$(pwd)\"}}}"

    # Evaluate through Onus Core.
    local result
    result=$(printf '%s' "$json" | "$(onus_bin)" evaluate 2>/dev/null)
    local exit_code=$?

    if [ $exit_code -ge 2 ]; then
        # Blocked or escalated.
        local correction
        correction=$(printf '%s' "$result" | grep -o '"correction":"[^"]*"' | head -1 | sed 's/"correction":"//;s/"//')
        if [ -n "$correction" ]; then
            echo ""
            printf '\033[0;31m=== Onus BLOCKED ===\033[0m\n' >&2
            printf '\033[0;33m%s\033[0m\n' "$correction" >&2
            echo "" >&2
        else
            echo ""
            printf '\033[0;31m=== Onus BLOCKED this command ===\033[0m\n' >&2
            echo "" >&2
        fi
        # Add the command to the history so the user can recall/review it.
        history -s "$cmd" 2>/dev/null || true
        return 1
    fi

    if [ $exit_code -eq 1 ]; then
        # Warn.
        local correction
        correction=$(printf '%s' "$result" | grep -o '"correction":"[^"]*"' | head -1 | sed 's/"correction":"//;s/"//')
        if [ -n "$correction" ]; then
            printf '\033[0;33mOnus: %s\033[0m\n' "$correction" >&2
        fi
    fi

    return 0
}

# ============================================================
# Bash: use DEBUG trap + PROMPT_COMMAND to intercept commands.
# ============================================================
if [ -n "$BASH_VERSION" ]; then
    # Store the last command before execution.
    onus_last_command=""
    onus_last_ret=0

    onus_preexec() {
        # Read the command from history (the last entry is the current one).
        onus_last_command=$(history 1 | sed 's/^ *[0-9]* *//')
    }

    onus_precmd() {
        # Store the last exit code before we run.
        onus_last_ret=$?
    }

    # Install into DEBUG trap for preexec, PROMPT_COMMAND for precmd.
    if [ -z "$PROMPT_COMMAND" ]; then
        PROMPT_COMMAND="onus_precmd"
    else
        PROMPT_COMMAND="onus_precmd;${PROMPT_COMMAND}"
    fi

    # The DEBUG trap fires before each command. We evaluate and potentially skip.
    # We use a trick: set the last command and check it.
    # Actually for bash, the DEBUG trap is invoked before every command, including
    # those in PROMPT_COMMAND. A cleaner approach: use `preexec` pattern.

    # Override: history-based interception via PROMPT_COMMAND.
    # Simpler approach: intercept via `eval` override pattern.
    # For bash, we use a different approach: override `command_not_found_handle`
    # combined with a check on what's about to run.
    #
    # The simplest reliable approach: use `preexec_functions` if available (bash 5.2+),
    # or fall back to DEBUG trap.

    # Register preexec using either bash-preexec or DEBUG trap.
    # Clean approach: debug trap for preexec.
    # Save existing DEBUG trap.
    onus_saved_debug_trap=$(trap -p DEBUG 2>/dev/null || true)

    # shellcheck disable=SC2154
    trap 'onus_debug_trap' DEBUG

    onus_debug_trap() {
        # Get the command about to be executed from BASH_COMMAND.
        local cmd="$BASH_COMMAND"
        # Skip our own commands and no-ops.
        case "$cmd" in
            onus_eval*|onus_preexec*|onus_precmd*|onus_debug_trap*|onus_shell_start*|onus_shell_end*|PROMPT_COMMAND=*|trap*|history\ -*)
                return 0
                ;;
        esac
        # Skip prompt-related shell internals.
        case "$cmd" in
            \[*|echo\ *|cd\ *|export\ *|unset\ *|source\ *|.)
                return 0
                ;;
        esac

        # Evaluate if shell wrapper is enabled.
        if [ -n "$ONUS_SHELL_ENABLED" ] && [ "$ONUS_SHELL_ENABLED" = "1" ]; then
            onus_eval "$cmd"
            local ev=$?
            if [ $ev -ne 0 ]; then
                # We need to skip this command. Use the kill -INT trick.
                # Kill the current process to abort the command.
                kill -INT $$ 2>/dev/null
                # Give the shell a chance to reset.
                sleep 0
                # Exit the subshell if we can.
                exit 1
            fi
        fi
    }

elif [ -n "$ZSH_VERSION" ]; then
    # ZSH: use preexec and precmd hooks.
    onus_preexec() {
        local cmd="$1"
        [ -z "$cmd" ] && return

        # Skip our own internal commands.
        case "$cmd" in
            onus_eval*|onus_shell_start*|onus_shell_end*|source\ *onus*)
                return 0
                ;;
        esac

        if [ -n "$ONUS_SHELL_ENABLED" ] && [ "$ONUS_SHELL_ENABLED" = "1" ]; then
            onus_eval "$cmd"
            local ev=$?
            if [ $ev -ne 0 ]; then
                # In zsh, we can use the `command_not_found_handler` or
                # simply set a flag that the precmd will check.
                ONUS_SKIP_NEXT_COMMAND=1
                # Kill the command with SIGINT.
                kill -INT $$ 2>/dev/null
            fi
        fi
    }

    # Register the hooks.
    autoload -Uz add-zsh-hook
    add-zsh-hook preexec onus_preexec
fi
