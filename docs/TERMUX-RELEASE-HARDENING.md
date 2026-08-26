# Termux Release Hardening v1

## Supported user contract

Padma’s supported Android-Termux command target is exactly `$PREFIX/bin/padma`. After a successful source installation, these commands remain the public workflow:

```bash
padma --version
padma
padma file.pd
```

The repository source installer is `install-termux.sh`. It is inspectable Bash, uses the official `https://github.com/OfficialBiohub/padma-lang.git` checkout only, and builds the checked-out source with `cargo build --release --locked` before copying the verified binary to `$PREFIX/bin/padma`.

## Explicit actions

| Command | What it does | What it does not do |
|---|---|---|
| `bash install-termux.sh --check` | Validates Termux prefix, safe checkout location, and install target. | It does not install packages, contact the network, clone/pull, build, replace a binary, or uninstall. |
| `bash install-termux.sh` | Runs visible `pkg` prerequisites, clones or fast-forwards the clean official checkout, builds a locked release, validates it, and atomically replaces one binary. | It does not edit shell profiles, install `yt-dlp`, read credentials, start services, request Android permissions, or access browser/device/cloud providers. |
| `bash install-termux.sh uninstall` | Removes only the regular file `$PREFIX/bin/padma`. | It does not delete the source checkout, projects, Android files, shared storage, or any other executable. |

The installer accepts no target URL, command, package name, binary path, shell profile, token, or provider settings. It supports only the standard Termux prefix `/data/data/com.termux/files/usr` and a checkout inside `$HOME`; these restrictions prevent it from acting as a general system installer.

## Safe replacement and failure behavior

When an existing regular `$PREFIX/bin/padma` exists, the installer first copies it to a temporary backup in the same directory. It validates the staged release binary using `--version`, atomically replaces the command, then validates the installed target again. If the post-replacement check fails, the prior binary is restored. The installer refuses symlinks and non-regular files at the target rather than following or overwriting them.

An upgrade is intentionally refused if the existing checkout has local Git changes or its `origin` is not the official repository URL. Preserve your own changes by committing, stashing, or using a different source checkout before updating; the installer must not overwrite unreviewed local source.

## First install, upgrade, recovery, and uninstall

```bash
# First install: obtain an inspectable source checkout, then install it.
pkg install -y git
git clone https://github.com/OfficialBiohub/padma-lang.git "$HOME/padma-lang"
cd "$HOME/padma-lang"
bash install-termux.sh --check
bash install-termux.sh
padma --version

# Upgrade the same clean checkout.
cd "$HOME/padma-lang"
bash install-termux.sh --check
bash install-termux.sh

# If `padma` is not found in the current shell.
export PATH="$PREFIX/bin:$PATH"
hash -r
"$PREFIX/bin/padma" --version

# Remove only the installed command when you choose.
cd "$HOME/padma-lang"
bash install-termux.sh uninstall
```

If the direct `$PREFIX/bin/padma --version` command works but `padma --version` does not, the current shell has an unusual PATH. Open a new Termux shell or use the displayed `export PATH` command. If the direct target is missing, run the explicit install command from a fresh clean official checkout. Do not paste credentials, tokens, private URLs, or personal paths into an issue report.

## Optional dependencies

Padma itself does not install `yt-dlp`, browser tools, provider SDKs, database servers, or cloud credentials. The authorized media example separately documents its optional `yt-dlp` prerequisite; install it yourself only if you need that permitted workflow. The release installer does not perform provider, QPU, network API, browser, device, or payment actions.
