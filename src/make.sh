echo "\n==== compiling shader ====\n"
cd shaders
./makelib.sh
cd ..

echo "\n==== building for release ====\n"
cargo build --release

echo "\n==== running with time ====\n"
time ../target/release/pinv
