# Thoughts and ideas

## Errors

For errors, there's a need to cater to at least two paradigms, one for libraries and one for applications. For libraries, I think just `fn() int throws Error1, Error2, ...` is a good approach. It should enumerate all the possible errors, at least for the vast majority of cases.
For applications, the main goal is to be able to ask questions about what went wrong after the fact. That means at least knowing where the error happened, context, and the path the code took to get there (perhaps just a backtrace, but perhaps it's too verbose. Maybe only need stuff from the package). Application errors need at least the following
- A message to show to users.
- Context, for internal use, to pinpoint what went wrong
- Source location
- A simple way to add additional information like http endpoint, and such
This could prob just be something like a std.error.Error that works sort of like anyhow, but with the below addition.

I like the idea that error propagation should not be _too_ easy. Going from one error to another should explicitly add context and such.

This sounds a lot like tracing though, perhaps could play nicely together.

- Library errors
  - Consumers: Applications or other libraries
  - What do consumers want with it: Be able to do logic based on the error, e.g. retry the operation. i.e. the errors should be actionable
- Application errors
  - Consumers: End-users, devs debugging the code
  - What do consumers want with it:
    - End-users: The error should be actionable, but much less technical
    - Devs debugging: Context, source location, and so on

Blogs
- https://joeduffyblog.com/2016/02/07/the-error-model/
- https://fast.github.io/blog/stop-forwarding-errors-start-designing-them/ and original reddit post https://www.reddit.com/r/rust/comments/1q3wb3l/stop_forwarding_errors_start_designing_them/

## Things I like about other programming languages
Mostly languages I don't know that much already. Won't write about js/ts, rust

### Concurrency

- Lua: I sort of like Lua's coroutines. yield'ing values and suspending the function until further seems very nice and a simple interface. Seems to be basically like a generator, but a bit more powerful. Feels like they need function coloring, though, to distinguish them from other functions, so people don't just call them as is. I think a pitfall is that users might thing that this is parallelism, which it's not. Basically feels like a generator, probably more powerful.

### Parallelism

- Elixir: Quite like the simplicity of processes. Something like only allowing message passing seems to strike a good balance between allowing speed, and simplicity by not having mutexes and such. Easy in Elixir since all variables are immutable. Needs some checks in Fowl since we have mutable variables. Nice with spawn_link to link the processes, and make a spawned processes crash propagate to the parent. Should perhaps be the default. Like that you get a `caller` when you receive a message to send directly back to the process.
- Dart: Isolates go some of the right direction in that all data is isolated even though they have mutable variables. However it is very implicit that mutating global variables from an isolate only mutates it in that isolate. Also don't like that you have to await it, then you don't get rid of function colouring. Perhaps I don't really like isolates. The concept sounds right, but the execution is not quite there.

### Serialization

### Drop / defer

Let's take for instance how to manage a DB connection. It is very common to acquire a connection, call some queries, and then return the connection to the pool. Fowl needs an ergonomic way to handle these cases.

- Swift: I think you could use defer, not sure how postgresNIO does it internally.
- Rust: Really like the Drop trait, but really relies on knowing when variables are dropped. Feels out of place, perhaps, in a garbage collected language to hook into variable lifetimes.
- Python: with keywords could be the way, not sure I like them though.

### Other things

- Lua: Like that you can embed it, would be cool to make Fowl embedded, but sort of orthogonal to the current direction. Would perhaps fragment stuff a bit to both have an embedded mode, and a standalone mode that makes a binary, or what?
- Swift: I like named parameters, and that you can use `_` to make them positional. Not sure I want argument labels.
- Swift: I like the compile time check for API availbility for a given target.
- Python: I like list comprehension.
- Elixir/erlang: Atoms is an interesting idea. Sort of like it, but feels like they need it since they don't have enums/types.
- Consider dropping if else and go with just match like gleam. Keeps the lang simpler perhaps?

## Quickfire language features and things that should be possible / nice in the language
Things that are not mentioned above mostly

- `Drop`-like feature (perhaps `Descope`), that would make it simple to e.g. ROLLBACK a transaction if you forget it. Maybe this has something to do with the below point
- Move semantics of some kind. For instance, giving a transaction to a spawned task should move it into that task, in the sense that the type-system should tell the user that the current scope can no longer use it. Also for something like response bodies, where consuming it twice is a logic error.
- Conditional compilation
- Nice for frontend?
  - Perhaps something like xml-like function call syntax
- Remote debugging?
- sqlx-like typesafe sql queries
- Simple generation of openapi schema without all the boilerplate
- Simple serialization experience
- Operator overloading, is that what it's called. e.g. defining `+` for some type
- Perhaps loops as blocks that return new iterators, e.g. `let a = for i in range(0, 3) { 2 * i }; // a = [0, 2, 4]`
- Configurable garbage collection? Different strategies might work better for certain tasks
- Compiler should know if a closure is mut, so that task.spawn can't take mutable closures.
- Extendable linting by libs.
- Hard to go wrong for juniors, easy to do right for seniors.
- Excellent debugging experience
- Very nice in production

## Interesting blog posts

- [Language Design: Stop Using <> for Generics](https://soc.me/languages/stop-using-angle-brackets-for-generics)
- Language Design: Unified Condition Expressions: https://soc.me/languages/unified-condition-expressions and https://soc.me/languages/unified-condition-expressions-implementation
