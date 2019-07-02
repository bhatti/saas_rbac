rm db.sqlite
diesel setup
diesel migration redo
diesel migration run
diesel print-schema > src/plexrbac/persistence/schema.rs
