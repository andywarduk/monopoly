# Monopoly Probabilities

## WASM version:

[Go to the live page](https://andywarduk.github.io/monopoly/).

rust, cargo and wasm-pack need to be installed to build the WASM version.

To build an html directory containing individual assets and open it in the default browser:

```bash
cd monopoly-wasm
./build.sh -o
```

To build a stand alone html page (index.html) and open in the default browser:

```bash
cd monopoly-wasm
./build.sh -s -o
```

or run the convenience script:

```bash
./wasm.sh
```

## Console version:

![Console version](./screenshots/Screenshot-tui.png)

rust and cargo need to be installed to run this version.

```bash
cargo run --bin monopoly-tui --release
```

or run the convenience script:

```bash
./tui.sh
```

## Viewing probability matrices:

Run the following:

```bash
cargo run --bin monopoly-calc --release
```

or run the convenience script:

```bash
./stats.sh
```

csv files are put in 'csv' directory and a probabilities.xlsx is produced.

## Credits

[http://www.tkcs-collins.com/truman/monopoly/monopoly.shtml](http://www.tkcs-collins.com/truman/monopoly/monopoly.shtml)

[https://www.diva-portal.org/smash/get/diva2:1471765/FULLTEXT01.pdf](https://www.diva-portal.org/smash/get/diva2:1471765/FULLTEXT01.pdf)
