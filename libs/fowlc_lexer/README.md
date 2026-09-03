# Fowl lexer

The current lexer is generated using [re2rust](https://re2c.org/manual/manual_rust.html). We might want to build our own at some point, but this generates a very fast lexer, and it's quite easy to use.

## Generating the lexer

To generate the lexer, modify the src/lexing.re syntax file, and the run
```
re2rust src/lexing.re --output src/lexing.rs --no-unsafe --start-conditions --no-generation-date --no-version
```

the `--no-unsafe` block being there mostly to remove the unneeded unsafe blocks generated (none, it seems to me, are actually in need of unsafe), though we might want to consider removing the flag again if the unsafe blocks are not redundant and generate a faster lexer, perhaps without bounds checks.
