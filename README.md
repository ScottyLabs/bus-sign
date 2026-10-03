# Bus Sign

A real-time bus sign that displays Pittsburgh Regional Transit (PRT) arrivals for the Forbes & Morewood bus stops. Made in collaboration with the [Undergraduate Student Senate](https://www.cmu.edu/stugov/senate/). Launched in the Cohon University Center, coming soon to the Tepper Building!

Visit the online bus sign at https://bus-sign.scottylabs.org!

## Prerequisites

- Be a member of [Community-Based Projects](https://git.cmu.dev/ScottyLabs/governance/src/branch/main/data/teams/cbp.toml)
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

Visit the frontend at http://127.0.0.1:5173/!
