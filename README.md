# Rust + Postgres Triple Store
A simple Triple Store made with Postgres, Rust and Diesel
intended to be used with a postgres instance given by either setting an .env file or DATABSE_URL in default postgres format

## Setup dev env
this is for using podman with postgres, requires podman to be installed
```sh
clone https://github.com/enbyio/pg-triple-store.git
cd pg-triple-store
chmod +x start_podman.sh
POSTGRES_PASSWORD=yourpassword ./start_podman.sh
echo "DATABASE_URL=postgres://postgres:yourpassword@localhost:5432/triple_store" > .env
```

## use library in project
put in your Cargo.toml
```toml
[dependencies]
pg-triple-store = { git = "https://github.com/enbyio/pg-triple-store.git"}
```
