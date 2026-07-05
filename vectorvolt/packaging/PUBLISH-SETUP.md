# Publishing Setup Guide

All six publish workflows are dormant until you push a release tag (`v0.1.0`).
Before that first tag, complete the one-time setup below.

## GitHub Repos to create (all under `workerruntime` org)

| Repo | Purpose |
|---|---|
| `workerruntime/vectorvolt` | Main source repo (move from sandeepbassioec/ekos when ready) |
| `workerruntime/homebrew-vectorvolt` | Homebrew tap — hosts `Formula/vectorvolt.rb` |
| `workerruntime/scoop-vectorvolt` | Scoop bucket — hosts `vectorvolt.json` |
| `workerruntime/apt-vectorvolt` | apt repo — GitHub Pages branch serves signed .deb files |
| `workerruntime/rpm-vectorvolt` | yum/dnf repo — GitHub Pages branch serves signed .rpm files |
| `workerruntime/winget-pkgs` | Fork of `microsoft/winget-pkgs` — workflows open PRs from here |

## Secrets to add to the main vectorvolt repo

| Secret | Workflow | How to get it |
|---|---|---|
| `TAP_DEPLOY_TOKEN` | homebrew, scoop, apt, rpm | GitHub PAT with `repo` scope on the workerruntime org |
| `WINGET_TOKEN` | winget | GitHub PAT with `repo` scope, must have fork of microsoft/winget-pkgs |
| `CHOCOLATEY_API_KEY` | chocolatey | Create account at chocolatey.org → API Keys page |
| `APT_GPG_PRIVATE_KEY` | apt | GPG key for signing .deb files (see below) |
| `APT_GPG_PASSPHRASE` | apt | Passphrase for the GPG key |
| `RPM_GPG_PRIVATE_KEY` | rpm | GPG key for signing .rpm files (can be same key as apt) |
| `RPM_GPG_PASSPHRASE` | rpm | Passphrase for the RPM GPG key |

## Generating a GPG signing key

```bash
gpg --batch --gen-key <<EOF
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: WorkerRuntime
Name-Email: hello@workerruntime.com
Expire-Date: 0
Passphrase: YOUR_PASSPHRASE
EOF

# Export private key for GitHub secret
gpg --armor --export-secret-keys hello@workerruntime.com
```

Copy the full `-----BEGIN PGP PRIVATE KEY BLOCK-----` output into `APT_GPG_PRIVATE_KEY` and `RPM_GPG_PRIVATE_KEY` secrets.

## Enable GitHub Pages

For `apt-vectorvolt` and `rpm-vectorvolt` repos:
- Settings → Pages → Source: `gh-pages` branch (or `main` if that's where the workflow pushes)

## User install commands (after first release)

### macOS / Linux (Homebrew)
```bash
brew tap workerruntime/vectorvolt
brew install vectorvolt
```

### Windows (Scoop)
```powershell
scoop bucket add vectorvolt https://github.com/workerruntime/scoop-vectorvolt
scoop install vectorvolt
```

### Windows (Chocolatey)
```powershell
choco install vectorvolt
```

### Windows (winget)
```powershell
winget install WorkerRuntime.VectorVolt
```

### Debian / Ubuntu (apt)
```bash
curl -fsSL https://workerruntime.github.io/apt-vectorvolt/KEY.gpg \
  | sudo gpg --dearmor -o /usr/share/keyrings/vectorvolt.gpg
echo "deb [arch=amd64,arm64 signed-by=/usr/share/keyrings/vectorvolt.gpg] \
  https://workerruntime.github.io/apt-vectorvolt stable main" \
  | sudo tee /etc/apt/sources.list.d/vectorvolt.list
sudo apt update && sudo apt install vectorvolt
```

### RHEL / Fedora / Amazon Linux (yum/dnf)
```bash
curl -fsSL https://workerruntime.github.io/rpm-vectorvolt/KEY.gpg | sudo rpm --import -
sudo tee /etc/yum.repos.d/vectorvolt.repo <<EOF
[vectorvolt]
name=VectorVolt
baseurl=https://workerruntime.github.io/rpm-vectorvolt/
enabled=1
gpgcheck=1
gpgkey=https://workerruntime.github.io/rpm-vectorvolt/KEY.gpg
EOF
sudo dnf install vectorvolt
```

### Universal (curl pipe)
```bash
curl -fsSL https://raw.githubusercontent.com/workerruntime/vectorvolt/main/packaging/install.sh | bash
```
