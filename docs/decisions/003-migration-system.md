# Title: Migration System

## Context

Database management is a key trait in the creation of the tool. Modification of the Database
is something that has to be considered.

## Decision

Migrations were chosen for the versioning of the database schema and ensuring that it updates
the database every time a field is added or deleted from the database.

## Alternatives Considered

Leaving the database without migrations was also an option but it was discovered that I would
have to recreate the database multiple times every time I modify the schema, hence losing all
project data

## Consequences

This leaves the database upgradable and easier to manage in the long run.
