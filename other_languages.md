# Things I like about other programming languages
Mostly languages I don't know that much already. Won't write about js/ts, rust

## Errors

## Concurrency

- Lua: I sort of like Lua's coroutines. yield'ing values and suspending the function until further seems very nice and a simple interface. Seems to be basically like a generator, but a bit more powerful. Feels like they need function coloring, though, to distinguish them from other functions, so people don't just call them as is. I think a pitfall is that users might thing that this is parallelism, which it's not. Basically feels like a generator, probably more powerful.

## Parallelism

- Elixir: Quite like the simplicity of processes. Something like only allowing message passing seems to strike a good balance between allowing speed, and simplicity by not having mutexes and such. Easy in Elixir since all variables are immutable. Needs some checks in Fowl since we have mutable variables. Nice with spawn_link to link the processes, and make a spawned processes crash propagate to the parent. Should perhaps be the default. Like that you get a `caller` when you receive a message to send directly back to the process.
- Dart: Isolates go some of the right direction in that all data is isolated even though they have mutable variables. However it is very implicit that mutating global variables from an isolate only mutates it in that isolate. Also don't like that you have to await it, then you don't get rid of function colouring. Perhaps I don't really like isolates. The concept sounds right, but the execution is not quite there.

## Serialization

## Drop / defer

Let's take for instance how to manage a DB connection. It is very common to acquire a connection, call some queries, and then return the connection to the pool. Fowl needs an ergonomic way to handle these cases.

- Swift: I think you could use defer, not sure how postgresNIO does it internally.
- Rust: Really like the Drop trait, but really relies on knowing when variables are dropped. Feels out of place, perhaps, in a garbage collected language to hook into variable lifetimes.
- Python: with keywords could be the way, not sure I like them though.

## Other things

- Lua: Like that you can embed it, would be cool to make Fowl embedded, but sort of orthogonal to the current direction. Would perhaps fragment stuff a bit to both have an embedded mode, and a standalone mode that makes a binary, or what?
- Swift: I like named parameters, and that you can use `_` to make them positional. Not sure I want argument labels.
- Swift: I like the compile time check for API availbility for a given target.
- Python: I like list comprehension.
- Elixir/erlang: Atoms is an interesting idea. Sort of like it, but feels like they need it since they don't have types.
