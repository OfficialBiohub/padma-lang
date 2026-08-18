# Capability examples

These are complete Termux-friendly Padma projects. From the repository root, inspect a project before running it:

```bash
padma capabilities examples/capabilities/bangla-http
padma examples/capabilities/bangla-http
```

The first example uses Bangla source and grants only `network:http`. The second uses English source and grants only `filesystem:write`:

```bash
padma capabilities examples/capabilities/english-file-write
padma examples/capabilities/english-file-write
```

For a manifest-run project, `output.txt` is created relative to the project root, even when the command is started from another Termux directory. Project mode rejects `..`, symlink escapes, and `@downloads`; direct single-file scripts retain their documented compatibility path behavior.
