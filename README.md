# Monopoly Probabilities #

## WASM version: ##

![WASM version](./screenshots/Screenshot-wasm.png)

rust, cargo and wasm-pack need to be installed to build the WASM version.

To build an html directory containing individual assets and open it in the default browser:
```bash
cd monopoly-wasm
./Build.sh
```

To build a stand alone html page (single/index.htm) and open in the default browser:
```bash
cd monopoly-wasm
./BuildSingle.sh
```

## Console version: ##

![Console version](./screenshots/Screenshot-tui.png)

rust and cargo need to be installed to run this version.

```bash
cd monopoly-tui
cargo run --release
```

## Credits ##

[http://www.tkcs-collins.com/truman/monopoly/monopoly.shtml](http://www.tkcs-collins.com/truman/monopoly/monopoly.shtml)

[https://www.diva-portal.org/smash/get/diva2:1471765/FULLTEXT01.pdf](https://www.diva-portal.org/smash/get/diva2:1471765/FULLTEXT01.pdf)
