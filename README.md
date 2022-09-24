SpyCast is a crossplatform mDNS enumeration tool that can work either in active mode by recursively querying services, or in passive mode by only listening to multicast packets.

![spycast](https://i.imgur.com/E6n3Xwl.png)

## Building

```sh
cargo build --release
```

OS specific bundle packages (for example dmg and app bundles on OSX) can be built via:

```sh
cargo tauri build
```

SpyCast can also be built without the default UI, in which case all output will be printed on the terminal:

```sh
cargo build --no-default-features --release
```

## Running 

Run SpyCast in active mode (it will recursively query all available mDNS services):

```sh
./target/release/spycast
```

Run in passive mode (it won't produce any mDNS traffic and only listen for multicast packets):

```sh
./target/release/spycast --passive
```

## Other options

Run `spycast --help` for the complete list of options. 

## License

This project is made with ♥  by [@evilsocket](https://twitter.com/evilsocket) and it is released under the GPL3 license.