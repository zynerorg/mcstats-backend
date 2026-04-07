# Minecraft-stats

A rust project that watches a minecraft world folder and serves different API endpoints for the stats of the players of that world.

## Setup

Clone this repo and use docker to deploy.

```bash
docker compose up -d
```

## Development

Clone this repo and cd into it.

```bash
docker compose up db
```

Note: if wished you could also add on the "-d" flag to run the db in detached mode.

Then open another terminal and use "cargo run" to test and iterate on the project.
