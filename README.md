# Bus Sign

A real-time bus sign for the Cohon University Center at Carnegie Mellon University.

## Initial Setup

1. Create a `.env` file in the project root directory, following the format of `.env.example`. Note that you will need a PRT API key to run this project.

2. Install `npm`.

3. Install relevant dependencies:
`npm install --prefix frontend`

## Running the Project

Navigate to the project's root directory, then run the following commands.

**Frontend**

`npm run dev --prefix frontend`

**Backend**

`cargo run --manifest-path backend/Cargo.toml`
