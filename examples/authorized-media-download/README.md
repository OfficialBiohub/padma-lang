# Authorized media download

This example invokes the locally installed `yt-dlp` executable through Padma's `media.download` contract. Use it only for media you own or are authorized to download, and comply with the relevant platform terms.

Run `padma .` after installing `yt-dlp` in Termux. The selected file is written to this project directory with a `video-<id>.<ext>` filename. Project mode deliberately does not write directly to Android shared storage.
