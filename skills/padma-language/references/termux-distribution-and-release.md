# Termux Distribution and Release

## Read This Reference When

Read this file for installation, packaging, release binaries, user setup, or GitHub distribution changes.

## User-Facing Termux Contract

Padma is designed to work on an Android phone through Termux. Preserve a simple path from installation to these commands:

```bash
padma --version
padma
padma file.pd
```

Keep `$PREFIX/bin/padma` as the installed command target. Update `install-termux.sh`, packaging notes, and the root README together when installation prerequisites or output location changes.

## Distribution Rules

1. Prefer an inspectable clone/build installer path over undocumented setup magic.
2. Build release mode with locked dependencies before calling a release ready.
3. Keep the project runnable without a desktop-only dependency or a mandatory web playground.
4. State external prerequisites exactly, such as `sqlite`, `yt-dlp`, `openssl`, or a provider token; do not bundle secrets or claim a system tool is built in.
5. Test an example from a temporary project copy so generated database/site/download files do not pollute the tracked example directory.

## Release Documentation

Update the relevant README and `CHANGELOG.md` for visible behavior. State whether a feature is local-only, planning-only, experimental, or requires explicit provider/user confirmation. Do not label a build as production-safe unless all current release gates pass.
