# ONUS PROVIDER CONFIGURATION

Generated: 2026-06-18

## Supported Provider Types

Onus supports multiple semantic providers for action evaluation. The provider is configured
via environment variables read by the daemon at startup.

### Provider types (from `src/semantic.rs`):

| Provider kind | Description | Use case |
|---------------|-------------|----------|
| `disabled` | No semantic evaluation; only static rules | Testing, offline |
| `deterministic` | Built-in rule-based evaluation (no external LLM) | Offline/air-gapped |
| `local` | Local subprocess (e.g., llama.cpp, ollama) | Privacy-sensitive |
| `cloud` | Remote OpenAI-compatible API | Full-featured |
| `fixture` | Test-only deterministic fixture | Development/testing |

---

## Environment Variables

Copy `config/examples/onus.env.example` to your environment and fill in placeholders.
Do **not** commit the filled-in file.

| Variable | Valid values | Required | Description |
|----------|-------------|----------|-------------|
| `ONUS_SEMANTIC_PROVIDER` | `disabled`, `deterministic`, `local`, `cloud` | Yes | Provider selection |
| `ONUS_API_KEY` | `<your-api-key>` | For `cloud` | API key (never logged) |
| `ONUS_API_ENDPOINT` | URL | For `cloud` | e.g. `https://api.openai.com/v1` |
| `ONUS_MODEL` | Model name | For `cloud` | e.g. `gpt-4o` |
| `ONUS_LOCAL_COMMAND` | Command path | For `local` | e.g. `ollama run llama3` |
| `ONUS_TIMEOUT_MS` | Integer (ms) | No (default: 5000) | Provider response timeout |
| `ONUS_MAX_INPUT_BYTES` | Integer | No (default: 16384) | Max semantic input size |
| `ONUS_TOKEN_BUDGET` | Integer | No (default: 4000) | Max semantic output tokens |
| `ONUS_PRIVACY_MODE` | `strict`, `balanced`, `off` | No (default: `balanced`) | Payload redaction |
| `ONUS_DATA_DIR` | Directory path | No (default: `~/.onus`) | Onus data + database location |
| `ONUS_LISTEN_PORT` | Integer (port) | No (default: 4837) | Daemon listen port |

## Security invariants

- **`ONUS_API_KEY` is never logged** — the daemon redacts it from all output, receipts, and traces.
- **`ONUS_PRIVACY_MODE=strict`** redacts sensitive fields from payloads before semantic evaluation.
- **Fallback**: If `cloud` provider fails, the daemon falls back to `deterministic` mode.
- **Timeout**: Default 5 seconds. Configurable via `ONUS_TIMEOUT_MS`.

## Connection test

Run `onus doctor` — it checks provider configuration as part of the diagnostics.
Or run `onus status` with the daemon started.

## Privacy behavior by mode

| Mode | Payload sent to LLM | Secrets in receipts | Sensitive fields in logs |
|------|-------------------|--------------------|--------------------------|
| `strict` | No (hashing only) | Never | Never |
| `balanced` | After redaction | Never | Never |
| `off` | Full payload | Never | Never |

## Provider configuration files

No provider configuration file is read. All configuration is through environment variables.
This prevents credential leakage through config files committed to version control.
