# Milestone 2: Server-client with WebAssembly

In this assignment, you will port your implementation from milestone 1 into a
server and a client, and adjust a communication protocol.  Although, the overall
structure for the project is given to you, you still need to figure out how to
extend the systems you build to work with multiple clients' states without
interference.

While working on this assignment, you will:

1. Work on a repo over a longer term than most other courses, and have a
   dialogue with other developers (your TAs and instructor) through code
   reviews, like milestone 1.
3. Work on some basic WebAssembly development.
4. Test and debug software consisting of multiple programs with different
   targets (the native server and the WebAssembly client).
4. Get experience with network programming using Lightyear.

You need to _pass_ this assignment in order to qualify for a C or above, and in
order to move onto the next milestone.

Based on the lines I removed from my implementation, you will need to write
300--350 lines of code, most of this will be porting your existing code.

## Deadlines

- Soft deadline: Monday, April 13 (right before the Spring break).
- Hard deadline: Monday, April 20.
    - This is the last day for you to **pass all code reviews** and schedule an
      interactive grading session.
    - Note: The gap between soft and hard deadlines will shrink for future
      assignments because there is a hard deadline imposed by the university at
      the end of the semester.

## Setup

You need to make sure that you have the following installed:

- wasm-bindgen-cli
  - Make sure your version of wasm-bindgen-cli matches the version of
    wasm-bindgen in `Cargo.toml`.
- wasm-pack
- Optional but recommended: Firefox. I will run client tests on Firefox.

## Testing

**This section will be filled once the test suite is released.**

There are no client tests.  You will show it via a live demo during interactive
grading.

There are tests for the server that you can run under the `server/` directory
using `cargo nt`.  These tests check only if a single client is handled
properly.  You should test with multiple clients, as that will be part of your
demo.

### Running the tests

You can run the server tests via `./test-server.sh`.  Alternatively, you can run
`cargo nt -p server` directly.

### Debug client and debug server

You can download a debug client from the course website to debug your server.
This is my client implementation.

You can also download a debug server, this is only for Linux (check the course
website on whether there is a macOS version released) because it is just the
compiled version of my server.

## Directory structure

```
.
├── Cargo.lock
├── Cargo.toml
├── client                 # Client code, compiled to wasm32
│   ├── Cargo.toml
│   └── src
│       ├── board.rs       # Input handling and rendering code
│       ├── lib.rs         # Client library entry point, just some helpers
│       ├── main.rs        # The main program, sending messages, etc.
│       └── ui.rs          # UI rendering code
├── CODEOWNERS
├── common                 # Data types and logic shared between the two programs
│   ├── Cargo.toml
│   └── src
│       ├── bag.rs         # RNG bag logic
│       ├── board.rs       # Board data structures
│       ├── config.rs      # Game configuration (fused for client and server for simplicity)
│       ├── data.rs        # Core data structures
│       ├── lib.rs         # Library entry point, has `there_is_collision` and some constants.
│       └── protocol.rs    # The communication protocol
├── deploy-client.sh
├── README.md
├── run-game-server.sh
├── run-web-server.sh
├── server                 # Server code
│   ├── Cargo.toml
│   ├── config.json        # Default game configuration
│   └── src
│       ├── game_logic.rs  # Core game logic, must support multiple clients
│       ├── lib.rs         # Helpers for writing server-side game logic
│       ├── main.rs        # Main server binary
│       ├── net.rs         # Networking code for the server, handles initializing clients and firing input events
│       ├── score.rs       # Score event handler
│       ├── tests.rs       # Basic server unit tests
│       └── tests/         # Additional test code
└── site
    ├── config.json -> ../server/config.json
    └── index.html
```

The three important scripts are:

- `deploy-client.sh`: Compiles the client and deploys it to `site/`.
- `run-web-server.sh`: Runs a web server to serve at `site/`.
  - You don't need to restart the server after compiling the client. You can
    just force-refresh the tab.
- `run-game-server.sh`: Runs the game server.

You should run all three of them from the repository root.

Detailed structure is covered in lectures, watch relevant lecture videos.

I will release the full test suite later in the week, be on the lookout for
that.  Those tests will be locked behind Cargo features.

## What you need to implement

Rather than using Cargo features, we will use other flags to control different
tests.  You don't need to enable Cargo features.  You will turn in your
implementation in two stages: the client, and the server.

## Client

- Mostly copying over your code from milestone 1.  Need to adjust input handling
  to send a message, and some of the redrawing system.
- Declare objects that will be replicated, communication channels, and messages
  in `common/src/protocol.rs`.

You can test against the debug server once it is released.  You will test the
client by a live demo during interactive grading.  There will be minimal
automated tests on the client.

## Server

- You need to implement some ECS and networking helpers in `server/src/lib.rs`
  and use them to select the data relevant to each client when porting your
  systems from milestone 1.
- Copy over your systems to `game_logic.rs` and `score.rs`.
- Add the initialization code to relevant parts of `main.rs` to create the
  "resources" for a client on connection.
- Your code must work with multiple clients.

## Implementing and submitting a step

You need to work on each feature (the client, the server) in a separate branch
(you can do so concurrently if you don't want to wait for code reviews), and
create 1 PR per feature.  Each feature needs to have the following workflow:

1. Implement the feature on its own branch and make sure it passes all tests.
   - Write additional tests if the feature spec requires you to do so.
   - *Commit and push regularly.*
2. Create a PR for the feature.  **All your code and PR description must be
   professional.** In your PR description, give an overview of the structure
   and behavior of your code, and any tests you have added.
   - GitHub will assing a reviewer automatically.
3. Make sure the code passes the tests on the CI server too.
4. Go through the code review process until you get approval.
5. Merge the feature.

## Changes from Milestone 1

- We don't maintain separate plugins for hard drop, score, hold, etc.  You will
  port most of the game in one fell swoop.
- The resources we want to replicate (`GameState`, `LockdownTimer`) are now
  components of a singleton entity.  Relevant systems should access these via
  `Single` on the client side, and `Query` on the server side, with a filter on
  the query to fetch a specific client's data.
  - `GameConfig` is still a resource, which means that a server can run only one
    type of game at a time, and that the server and the client need to have
    matching configurations.  We could have the same component treatment for
    `GameConfig` to an extent but we would still need to get the server address
    and port in a fixed configuration.  Having everything bundled for
    `GameConfig` simplifies the code enough that we are taking this shortcut for
    this class project.
- We split the game state into multiple parts (shared vs. private), depending on
  whether the relevant data can be send over the network.  An important part is
  that the RNG cannot be serialized, so the next tile bag is part of the private
  (server-only) state.
- `Board` is removed, we maintain the tiles on the main board just like the
  tiles on side boards.
- There may be multiple `Active` and `Next` tetrominoes on the server side, so
  the server needs to use the ones specific to each client separately.  See the
  in-class lightyear example for how to do this efficiently by building an
  inverse look-up table.
- For bags, we use an ADT instead of inheritance so that we can
  serialize/deserialize them without resorting to complex reflection tricks.
- Many game logic systems now use queries and need to filter entities relevant
  to each client **efficiently**.
- To support multiple clients, the signature of `there_is_collision` is more
  flexible, and it accepts arbitrary block iterators.

## What you are already given

Besides the overall structure, you are given the bag implementations,
configuration implementation, and `data.rs` along with some systems.

You are given the `gravity` system as an example for extending your systems to
work on multiple clients.

## Caveats

### Tab suspension

When testing the game, if you put the browser in the background (e.g., in a
different desktop), or switch tabs; the tab the client is in will be suspended
and you may lose connection.  Beware of this when debugging.

### wasm-bindgen version mismatch

If you get the following error, update your `wasm-bindgen-cli` version following
the instructions:

```
Generating WebAssembly bindings...
error: 

it looks like the Rust project used to create this Wasm file was linked against
version of wasm-bindgen that uses a different bindgen format than this binary:

  rust Wasm file schema version: 0.2.115
     this binary schema version: 0.2.114

Currently the bindgen format is unstable enough that these two schema versions
must exactly match. You can accomplish this by either updating this binary or
the wasm-bindgen dependency in the Rust project.

You should be able to update the wasm-bindgen dependency with:

    cargo update -p wasm-bindgen --precise 0.2.114

don't forget to recompile your Wasm file! Alternatively, you can update the
binary with:

    cargo install -f wasm-bindgen-cli --version 0.2.115

if this warning fails to go away though and you're not sure what to do feel free
to open an issue at https://github.com/wasm-bindgen/wasm-bindgen/issues!
```
  
## AI use policy

The following uses of generative AI tools are OK for this milestone:
- Debugging a particular compiler error,
- Generating **small** code snippets as long as you have proper attribution (see
  the next bullet point),
  - If you aren't sure what _small_ is, reach out to the professor for
    confirmation
- Generating test cases for an implementation (any AI-generated or other
  external code must have proper attribution, e.g. via a comment `//from:
  <source>`), and
- Asking an AI tool "how do I do this?" or "what does this part of the code
  do?".

Most of the code you submit as part of your implementation (i.e., anything that
is not a test) must be produced directly by you, and anything you haven't
produced directly must have proper attribution.  The AI use policy will get
relaxed over time, this is to ensure that you build a solid baseline before
using such tools so that you can thrive in this class rather than getting stuck
later.

## Grading

To pass this assignment, you need to:

1. Pass all the tests for all features,
2. Have your code pass `cargo fmt --check` and `cargo clippy`.
3. Pass code review.
4. Pass interactive grading.

For the code review, you have to work on a **non-main** branch and open a PR
when you want to test your implementation against GitHub actions.

### Code review

After you are done with step 2, you need to request code review.  The hard
deadline for this assignment is the deadline to **get approval and merge all
your PRs**.

After you are assigned a reviewer, they will review your code and request
explanations and/or changes.  Once you satisfy all these, yuo pass the code
review.  After you are done witch each set of changes, you need to ping the
reviewer on GitHub to ask for another round of code review until they approve
your changes.

### Interactive grading

After passing the code reviews, you have to go through an interactive grading
session.  This will be a 30-minute interview (i.e., oral exam) where you (1)
explain how your code works, the design decisions you made, and what approaches
you tried have not worked and (2) do a live demo of your game.

If you fail interactive grading once, you can do it again with me.  If you fail
it twice, you fail the whole assignment.  The objective of the assignment is to
show that **you have developed a certain understanding** and interactive
grading is where you will be able to demonstrate that.

