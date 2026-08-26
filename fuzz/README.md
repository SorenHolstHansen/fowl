# Fowl fuzzing test suite

## Running

To run the fuzzer, run (from the root)
```
cargo afl build -p fowl-fuzz
# And then
cargo afl fuzz -i fuzz/corpus -o fuzz/artifacts target/debug/fowl-fuzz
```
and to reproduce, run
```
cargo afl run fowl-fuzz < fuzz/artifacts/default/crashes/<crash_file>
```
