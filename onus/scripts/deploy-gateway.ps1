param(
    [Parameter(Mandatory = $true)]
    [string]$Server,
    [string]$User = "root",
    [int]$Port = 22,
    [string]$RemoteDir = "/opt/onus-gateway",
    [string]$EnvFile = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$gatewayDir = Join-Path $repoRoot "onus\apps\onus-gateway"
$archive = Join-Path $env:TEMP "onus-gateway.tar.gz"
$remote = "$User@$Server"

if (-not (Test-Path $gatewayDir)) {
    throw "Gateway directory not found: $gatewayDir"
}

tar -czf $archive -C (Join-Path $repoRoot "onus\apps") "onus-gateway"
scp -P $Port $archive "${remote}:/tmp/onus-gateway.tar.gz"
ssh -p $Port $remote "mkdir -p '$RemoteDir' && tar -xzf /tmp/onus-gateway.tar.gz -C '$RemoteDir' --strip-components=1 && cd '$RemoteDir' && npm ci --omit=dev"

if ($EnvFile) {
    if (-not (Test-Path $EnvFile)) {
        throw "Env file not found: $EnvFile"
    }
    scp -P $Port $EnvFile "${remote}:$RemoteDir/.env"
    ssh -p $Port $remote "chmod 600 '$RemoteDir/.env'"
}

Write-Host "Gateway uploaded to $remote:$RemoteDir"
Write-Host "On the VPS, install the systemd unit if desired:"
Write-Host "  sudo useradd --system --home $RemoteDir --shell /usr/sbin/nologin onus || true"
Write-Host "  sudo chown -R onus:onus $RemoteDir"
Write-Host "  sudo cp $RemoteDir/systemd/onus-gateway.service /etc/systemd/system/onus-gateway.service"
Write-Host "  sudo systemctl daemon-reload"
Write-Host "  sudo systemctl enable --now onus-gateway"
