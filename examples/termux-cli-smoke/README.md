# Termux CLI Smoke Test

This small project verifies the installed Padma release binary through the ordinary project command. It needs no capability grant, network, file output, browser, device action, or external tool.

```sh
cd ~/padma-lang
cargo build --release --locked
cd examples/termux-cli-smoke
../../target/release/padma .
```

Expected output:

```text
Padma Termux CLI ready
5
```

The program uses Bangla `ধরি` and `দেখাও`, interpolates a text variable, and evaluates Bangla digits. It proves only that this checked-out release binary can run one local `.pd` project; it does not test package-repository publishing, cloud services, browser/device control, or optional tools such as `yt-dlp`.
