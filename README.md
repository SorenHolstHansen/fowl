# fowl

> [!CAUTION]
> fowl is by no means ready for production use. Please use ironically only!

fowl is my pet programming language primarily used for learning purposes.
The basic features are / should be

- Readable / Reviewable and explicit. Code is hard to read, so whatever we can do to alleviate that is good, and it should not hide stuff behind magic.
- Simple memory management (probably GC, but not settled)
- Elixir-like concurrency
- Static typing.
- Great error handling experience.
- Very fast to compile
- Decent performance (think js or better)
- Great debugging experience

## Example

I provide no guarantees that this code will work, but this is what i think fowl _should_ look like (at the time of writing)

```fowl
use std.format.ToString

enum Category(
  Breaking
  Sport(/* which sport */ string)
)

struct Post (
  title: string
  length: int
  rating: float
  published: bool
  /* Anonymous struct */
  author: struct (
    name: string
  )
  category: Category
)

on Post fn publish(mut self) {
  self.published = true
}

struct Posts (
  posts: Array[Post]
)

enum DeleteError {
  PostNotFound
  CantDeleteBriansPosts
  Other(string)
}

/* ToString is an interface from the std lib */
impl ToString on DeleteError fn to_string(self) string {
  if self is {
    PostNotFound { "Post not found" }
    CantDeleteBriansPosts {
      "Brian doesn't like it when you delete his posts"
    }
    Other(s) {
      s
    }
  }
}

on Posts fn delete(
  mut self,
  /*
  params are named, which means you have to call this with `posts.mut.delete(index = ...)`.
  You could make it anonymous by writing `_ index: int`
  */
  index: int
) void throws DeleteError {
  if self.posts.len() > index {
    let post = self.posts.get(index)
    if post.author.name.starts_with("Brian") {
      throw DeleteError.CantDeleteBriansPosts
    }
    self.posts.mut.delete(index)
  }

  throw DeleteError.PostNotFound
}

fn main() {
  let mut post = Post(
    title = "Bad news indeed!"
    length = 1024
    rating = 5.0
    published = false
    author = (name: "Jon Smith")
    category = Category.Breaking
  )

  /* mutable access is explicit */
  post.mut.publish()
  

  let mut posts = Posts(posts = Array(post))

  let to_delete = 1
  try posts.mut.delete(index = to_delete) catch e {
    eprintln("Couldn't delete post {to_delete}, got {e}")
  }
}

/*
# TASKS
Tasks are like in elixir.
You don't need to add an `async` keyword to your functions, everything is basically async.
However tasks have some rules.
You can't pass mutable functions to tasks, since tasks should not mutate shared data.
They communicate with other processes by sharing messages.
Think of tasks like actors
*/

use std.tasks.spawn

enum MessageKind {
  Foo
  Bar
}

struct Message {
  kind: MessageKind
  body: string
}

fn task_example() {
  /* This does not compile as is mutates posts */
  /* spawn(fn() { posts.mut.delete(index = 2) }) */

  /* spawn a process, and use Message as the message protocol */
  let handle = spawn[Message](fn(receive: Receiver[Message]) {
    /* .. */
    let message = receive()
    if message.kind == MessageKind.Foo {
      println(message.body)
    }
    /* .. */
  })

  handle.send(Message(kind = MessageKind.Foo, body = "Hello"))
}
```

## License

Fowl is licensed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.
