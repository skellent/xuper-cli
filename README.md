# xuper-cli

Early Linux terminal player for Xuper/Magis TV. The bundled APK has not yielded a working Xuper catalog or playback endpoint yet. The commands below work with an explicit JSON catalog or M3U playlist containing direct stream URLs.

## Build

```sh
cargo build --release
```

The executable is `target/release/xuper-cli`. Install `mpv` to play streams.

## Use

```sh
./target/release/xuper-cli --url 'https://your-server.example/catalog'
./target/release/xuper-cli --url 'https://your-server.example/catalog' --random
./target/release/xuper-cli --url 'https://your-server.example/catalog' --search
./target/release/xuper-cli --file playlist.m3u --search
./target/release/xuper-cli --play 'https://your-server.example/video.m3u8'
```

Search filters titles as you type. Use Up/Down to select, Enter to play, and Esc to exit. An optional complete Authorization header can be provided through `XUPER_AUTHORIZATION` for catalog requests.

All catalog titles appear in search. `▶` means the entry has a direct stream URL; `·` means it only has metadata. Selecting a metadata-only title explains why playback is unavailable.

For example, the public TVMaze catalog used during development can be browsed with:

```sh
./target/release/xuper-cli --url 'https://api.tvmaze.com/shows?page=0' --search
```

TVMaze provides show metadata, not playback URLs. It is a search example and is not a Xuper catalog.

To try the terminal UI with a public demo video, run:

```sh
./target/release/xuper-cli --file examples/demo.m3u --search
```

## Xuper protocol findings

The APKs in `xuper/` are versions 4.34.4 and 4.99.20 of `com.android.mgstv`. Both are protected by packers. The newer APK's `assets/domain_test.json` still contains `xx` placeholders instead of server names. Its embedded strings name catalog routes such as `api/portalCore/v3/searchByName` and `api/portalCore/v10/startPlayVOD`, but do not provide a complete request format or a stable catalog host.

An app-filtered packet capture of version 4.99.20 on 2026-10-21 confirmed that playback calls `http://yuwc.swzablvpm.com/slb/v13/vod`, then requests `/vod/..._idx.idxg` and `/vod/..._media.ts` from the same host. The `.ts` request has signed parameters including `token`, `sign2`, and `expired`; the POST body and response are opaque encoded data. A small ranged request from Linux returned MPEG transport stream data, and `ffprobe` identified HEVC video and AAC audio. The captured link expires, so it is not a reusable catalog source.

The same capture saw HTTPS connections to `brkhzs.udigkkrc.com` and `tttopd.kj000zge.xyz` during catalog browsing. The encrypted requests prevent assigning a catalog route to either host yet. These observed domains may rotate and are not hardcoded into the CLI. The CLI still needs the app's catalog request and playback signing protocol to search Xuper directly.
