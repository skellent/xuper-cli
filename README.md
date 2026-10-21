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

To try the terminal UI with a public demo video, run:

```sh
./target/release/xuper-cli --file examples/demo.m3u --search
```

The APK is version 4.34.4 (`com.android.mgstv`). Its 13 KB `classes.dex` loads packed code, while `assets/domain_test.json` contains placeholder values. Static inspection has not exposed a usable media endpoint or a way to resolve an episode to a stream URL. The search and player flow can be connected to Xuper after that protocol is recovered.
