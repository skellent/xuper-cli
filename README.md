# xuper-cli

Early Linux catalog probe for Xuper/Magis TV. A working Xuper catalog endpoint has not yet been verified from the bundled Android APK, so the URL must be supplied explicitly.

## Build

```sh
cargo build --release
```

The executable is `target/release/xuper-cli`.

## Fetch a catalog

```sh
./target/release/xuper-cli --url 'https://your-server.example/catalog'
```

If the endpoint uses an Authorization header, set its complete value in `XUPER_AUTHORIZATION`. The CLI fetches JSON or M3U, then prints up to 20 media names. It does not play streams yet.

The current APK is version 4.34.4 (`com.android.mgstv`). Its small `classes.dex` is a loader for packed code, and `assets/domain_test.json` contains placeholder values, so neither provides a usable catalog endpoint. A redacted request and response captured from the app will let us implement the actual Xuper protocol.
