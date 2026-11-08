# Bus Sign

A real-time bus sign that displays Pittsburgh Regional Transit (PRT) arrivals for the Forbes & Morewood bus stops. Made in collaboration with the [Undergraduate Student Senate](https://www.cmu.edu/stugov/senate/). Launched in the Cohon University Center, coming soon to the Tepper Building!

Visit the online bus sign at https://bus-sign.scottylabs.org!

## Setup

Install Lix by following kennel's [local development guide](https://docs.kennel.scottylabs.org/guides/local-development/#installing-nix), then log in to OpenBao as described in its [secrets guide](https://docs.kennel.scottylabs.org/guides/secrets/#local-development). The PRT and OpenWeather API keys are stored there, so you need to be on the [Community-Based Projects](https://git.cmu.dev/ScottyLabs/governance/src/branch/main/data/teams/cbp.yaml) team, which you can [join through governance](https://git.cmu.dev/ScottyLabs/governance#joining-a-team).

Enter the dev shell and start the backend on :8080 and the frontend on :5173:

```bash
nix develop
pitchfork start --local
```

Visit the frontend at http://127.0.0.1:5173/!
