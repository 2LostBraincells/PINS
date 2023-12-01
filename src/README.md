# something something h1
## MSL library
### Compilation

In order to compile the .metal file into a usable .metallib library, run the makelib file in the terminal
```zsh
./makelib
```

#### makelib code

makelib.sh contains the following code:
```zsh
#!/bin/zsh

xcrun -sdk macosx metal -c compute.metal -o compute.air  
xcrun -sdk macosx metallib compute.air -o compute.metallib

rm compute.air
```