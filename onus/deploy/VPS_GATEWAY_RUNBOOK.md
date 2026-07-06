# Onus VPS Gateway Production Runbook

This runbook is for the private Onus semantic gateway. The public website can live on GitHub Pages or Vercel, but model-provider credentials must stay on the VPS.

Do not commit `.env` files or real secrets.

## 0. Production Shape

```text
User browser
  -> public site: https://ahsanmoizz.github.io/onus

User machine
  -> Onus CLI
  -> local console: http://127.0.0.1:3001
  -> local approvals: http://127.0.0.1:9191
  -> VPS gateway: https://YOUR_GATEWAY_DOMAIN/v1/chat/completions

VPS gateway
  -> OpenRouter / provider endpoint
```

## 1. Rotate Secrets First

Rotate any provider key, JWT secret, DB password, or admin token that was pasted into a chat, screenshot, terminal transcript, or issue.

Required private values:

```bash
ONUS_PROVIDER_API_KEY=rotated_provider_key
ONUS_DB_PASSWORD=long_random_database_password
ONUS_GATEWAY_PUBLIC_URL=https://api.example.com
```

Optional values:

```bash
ONUS_PROVIDER_ENDPOINT=https://openrouter.ai/api/v1/chat/completions
ONUS_PROVIDER_MODEL=qwen/qwen3-coder:free
ONUS_SITE_ORIGIN=https://ahsanmoizz.github.io
```

## 2. Upload Gateway From Windows

From PowerShell on your PC:

```powershell
cd D:\Onus
.\onus\scripts\deploy-gateway.ps1 -Server YOUR_VPS_IP -User root -RemoteDir /opt/onus-gateway
```

## 3. Bootstrap VPS

SSH into the VPS:

```bash
ssh root@YOUR_VPS_IP
```

Run:

```bash
cd /opt/onus-gateway
chmod +x scripts/vps/*.sh 2>/dev/null || true
```

Install Node 20, Postgres, nginx, and create the DB:

```bash
export ONUS_DB_PASSWORD='long_random_database_password'
bash /opt/onus-gateway/scripts/vps/bootstrap-gateway.sh
```

## 4. Create VPS `.env`

On the VPS:

```bash
export ONUS_PROVIDER_API_KEY='rotated_provider_key'
export ONUS_DB_PASSWORD='long_random_database_password'
export ONUS_GATEWAY_PUBLIC_URL='https://api.example.com'
bash /opt/onus-gateway/scripts/vps/create-gateway-env.sh
```

This writes:

```text
/opt/onus-gateway/.env
```

The file is chmod `600` and must remain only on the VPS.

## 5. Start Systemd Service

```bash
sudo cp /opt/onus-gateway/systemd/onus-gateway.service /etc/systemd/system/onus-gateway.service
sudo systemctl daemon-reload
sudo systemctl enable --now onus-gateway
sudo systemctl status onus-gateway --no-pager
```

## 6. Expose Gateway

### Option A: Cloudflare Tunnel

```bash
cloudflared tunnel --url http://127.0.0.1:8080
```

Use the HTTPS tunnel URL as `ONUS_GATEWAY_PUBLIC_URL` until you attach a real domain.

### Option B: nginx + Domain

Copy:

```text
onus/deploy/vps/onus-gateway.nginx.example
```

to:

```text
/etc/nginx/sites-available/onus-gateway
```

Replace `api.example.com`, then:

```bash
sudo ln -sf /etc/nginx/sites-available/onus-gateway /etc/nginx/sites-enabled/onus-gateway
sudo nginx -t
sudo systemctl reload nginx
sudo apt install -y certbot python3-certbot-nginx
sudo certbot --nginx -d api.example.com
```

## 7. Verify Gateway

```bash
bash /opt/onus-gateway/scripts/vps/verify-gateway.sh https://api.example.com
```

Expected:

```text
health: ok
ready: ok
auth guard: ok
```

## 8. Issue First Client Token

```bash
bash /opt/onus-gateway/scripts/vps/issue-client-token.sh admin-local-test https://api.example.com
```

The returned token is an Onus client token. It is not the OpenRouter/provider key.

## 9. Configure Local Onus Client

On Windows, edit:

```powershell
notepad "$env:APPDATA\Onus\onus.env"
```

Set:

```powershell
ONUS_STRICT=1
ONUS_MISSING_CONTRACT=block_mutating
ONUS_SEMANTIC_PROVIDER=cloud
ONUS_SEMANTIC_ENDPOINT=https://api.example.com/v1/chat/completions
ONUS_SEMANTIC_MODEL=onus-managed
ONUS_SEMANTIC_API_KEY=CLIENT_TOKEN_FROM_STEP_8
ONUS_SEMANTIC_FALLBACK=fail_closed
ONUS_SEMANTIC_FAIL_CLOSED_CRITICAL=1
ONUS_SEMANTIC_PRIVACY_MODE=strict
ONUS_SEMANTIC_REDACT=1
```

## 10. Run Local Onus

```powershell
onus doctor
onus start
onus status
onus intake --prompt "Fix the login bug and keep tests enabled."
```

## 11. Admin Panel

The admin console is local to the user's machine:

```powershell
onus console --port 3001
```

Open:

```text
http://127.0.0.1:3001
```

Approval UI:

```powershell
onus approvals serve --port 9191 --token YOUR_LOCAL_UI_TOKEN
```

Open:

```text
http://127.0.0.1:9191?token=YOUR_LOCAL_UI_TOKEN
```

Do not expose local console or approval ports publicly.

## 12. Release Gate

Do not tag a public release until:

1. `https://api.example.com/health` returns OK.
2. `https://api.example.com/ready` returns OK.
3. A client token can be issued.
4. Local Onus can complete a semantic-review request through the gateway.
5. The install docs/scripts no longer contain `YOUR-ONUS-GATEWAY`.
