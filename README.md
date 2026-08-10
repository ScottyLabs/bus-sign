# Bus Sign

## Prerequisites

- [devenv](https://devenv.sh/getting-started/) - provides Cargo, Deno, and other tooling via Nix
- PRT API Key - Obtained from creating a TrueTime account [here](https://realtime.portauthority.org/bustime/createAccount.jsp)
- OpenWeather API Key - Obtained from [OpenWeatherMap](https://openweathermap.org/api) (Current Weather + 5 Day / 3 Hour Forecast)

## Setup

### Setting up your environment variables

Secrets are managed with secretspec. Authenticate once per machine with:

```bash
nix run git+https://codeberg.org/ScottyLabs/kennel#login
```

Allow devenv, or enter the shell if already allowed:

```bash
devenv allow
# or: devenv shell
```

### Running the backend

```bash
# From the repo root, inside the devenv shell
cargo run -p backend
```

### Running the frontend

```bash
cd frontend

# Install dependencies
deno install

# Start the frontend
deno task dev
```
