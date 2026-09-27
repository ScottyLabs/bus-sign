# Bus Sign

## Prerequisites

- [devenv](https://devenv.sh/getting-started/) - provides Cargo, Deno, and other tooling via Nix

## Setup

### Secrets

The PRT and OpenWeather API keys are loaded from OpenBao by secretspec. Authenticate once per machine with:

```bash
nix run git+https://git.cmu.dev/ScottyLabs/kennel#login
```

Allow devenv, or enter the shell if already allowed:

```bash
devenv allow
# or: devenv shell
```

### Running

```bash
# Starts the backend on :8080 and the frontend on :5173
devenv up
```
