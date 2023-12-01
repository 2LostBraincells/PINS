# Swedish Personal Identity Number Validator

A hyper optimized program for validating Personal Identity Numbers (or PINS for short)

## Usage

### First time
To build the project make sure cargo and xcrun are installed then run these commands
```
cd src
./makelib.sh
cargo build
```

The compiled executable can then be found in `target/release/build` under the name `PINS`

### Benchmark
To get a rough benchmark run
```
cd src
./make.sh
```

This will compile the shaders, build the project and run it using the unix `time` command wich will output three different time values.
