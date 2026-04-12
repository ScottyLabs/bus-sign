# Bus Sign

## Prerequisites
- [Bun](https://bun.com/docs/installation) - JavaScript runtime and package manager
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) - Rust package manager and build system
- PRT API Key - Obtained from creating a TrueTime account [here](https://realtime.portauthority.org/bustime/createAccount.jsp)

## Setup
### Setting up your environment variables
```
# Copy env variables from .env.example
$ cp .env.example .env

# Add your PRT_API_KEY to the .env file
```
### Running the backend
```
$ cd backend

# Install dependencies and start the backend
backend $ cargo run
```
### Running the frontend

```
$ cd frontend

# Install dependencies
frontend $ bun install

# Start the frontend
frontend $ bun dev
```

