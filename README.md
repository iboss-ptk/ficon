# ficon
File and folder convention checker written in rust

[![asciicast](https://asciinema.org/a/rHTVDEyIvLPfC1DjlgeKnuCgM.svg)](https://asciinema.org/a/rHTVDEyIvLPfC1DjlgeKnuCgM)

## Install

The distribution is now available on [crates.io](https://crates.io/). You can install via `cargo`.

```
$ cargo install ficon
```

Or you can run through docker

```
$ docker run -v "$PWD":/usr/src/app ibosz/ficon
```

It will pull the ficon image from [dockerhub](https://hub.docker.com/r/ibosz/ficon)
and mount your current working directory to docker container to run `ficon`.

More options are coming. Contributions are welcome :)

## Usage

First you need to create `Ficon.toml` in the directory that you want to verify convention.

For basic usage, you can just have this configuration:

```toml
[default]
convention = "any"
```

And run
```
$ ficon
```

Everything will be green since we haven't put in any constraint.
Everything in `.gitignore` file will be ignored by default.


For more sophisticated example, this is copied from this project itself:

```toml
[default]
convention = "snake"

[[for_patterns]]
pattern = "*.toml"
convention = "pascal"

[[for_patterns]]
pattern = "*.md"
convention = "upper_snake"

[[for_patterns]]
pattern = "./LICENSE"
convention = "upper_snake"

[[for_patterns]]
pattern = "./Cargo.lock"
convention = "pascal"
```

You can specify default convention and convention for specific glob patterns using `[[for_patterns]]` as you can see above.
We are using `glob` crate to do glob matching, [see more](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).

The higher `[[for_pattern]]` position, the higher precedence it is, this behaviour might change in the next release.

## Convention

There are 6 predefined convention, namely
* `any` : no constraint
* `kebab` : kebab-case
* `snake` : snake_case
* `upper_snake` : UPPER_SNAKE_CASE
* `pascal` : PascalCase
* `camel` : camelCase

But if you want to define your own convention, you can do so by using regex:

```
[[for_patterns]]
pattern = "**/__*__/"
convention = "/^__[a-z]+__$/"
```

This will match path like `./test/app/__mock__` for example.

Note that `//` is required in order to use regex as convention.

The convention constraint only checks against file name or directory name, disregrading any extension,
for example, `src/lib/app.rs`, `ficon` only check if `app` matches the constraint.
If the file has multiple extension eg. `test/main/app.spec.ts`, again `ficon` will only check `app`.
