# Velvet

A repackage and republish of a combination of crates to create a specific web stack in a consistent and single point of view.
This is not meant to be a library with any specific purpose, only a short handing of boilerplate for the common setup and structure of this web stack.

## Stack used

Items of the stack:
  - WEB: Axum

## ENV vars
  - SERVER_BIND: ip for which to listen on
  - SERVER_PORT: port for which to listen on
  - SENTRY_URL: full url for sending data to sentry, if present
  - DATABASE_URL: postgres://user:pass@host:port/database (if database used)
  - DATABASE_MAX_CONNECTIONS: [number] (default 1)
  - STRUCTURED_LOGGING: true|false (default false)

