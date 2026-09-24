# faildigest

`cargo test` on any real project prints one line per test, plus compiler
warnings, doctests, and whatever the tests themselves write to stdout. When
a CI run fails you're usually looking at a log with a few thousand lines in
it and exactly three or four of them matter. faildigest answers one
question: which tests failed, out of everything in this output?

It reads test output from files you pass it, or from stdin if you pass
nothing, so it fits either end of a pipeline.

## usage

Pipe a live run straight through:

```
cargo test 2>&1 | faildigest
```

Or point it at a saved log:

```
cargo test 2>&1 | tee test.log
faildigest test.log
```

You can also pass several logs at once (say, one per CI shard) and it will
report across all of them:

```
faildigest shard-1.log shard-2.log shard-3.log
```

A bare `-` anywhere in the argument list means "read stdin here", so a
saved log can be combined with a live pipe:

```
cargo test -p other-crate 2>&1 | faildigest baseline.log -
```

Sample output:

```
2 failed, 14 passed, 1 ignored

FAILED  parser::tests::rejects_empty_input
FAILED  parser::tests::handles_trailing_newline
```

faildigest exits with status 1 if it found any failed tests, and 0
otherwise, so it can gate a CI step that only has a saved log to work
from.

## input format

Right now it understands the line shape libtest prints for each test:

```
test module::path::test_name ... ok
test module::path::test_name ... FAILED
test module::path::test_name ... ignored
```

Everything else in the input is skipped. This covers plain `cargo test`
output; other formats (JSON, JUnit XML, nextest's default renderer) aren't
read yet.

## building

Standard library only, no dependencies to fetch:

```
cargo build --release
```

## license

MIT, see LICENSE.
