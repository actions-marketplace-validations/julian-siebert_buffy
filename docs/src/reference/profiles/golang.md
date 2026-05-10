# Golang Profiles

The `golang` profile generates a Go module from your `.proto` files and
publishes it to a Git repository. Go modules are versioned by Git tags, so
the consuming side only needs `go get <module>@<tag>`.

Available variants:

* [`git`](#the-git-variant) --- Push the generated module to a Git remote.

## Required tools

* `protoc` --- Protocol Buffers compiler.
* `protoc-gen-go` --- Go code generator plugin.
* `protoc-gen-go-grpc` --- gRPC plugin (only when `grpc = true`).
* `go` --- Go toolchain (used for `go mod init`, `go mod tidy`, `go build`).
* `git` --- Used to commit, tag, and push the generated module.

`buffy check` verifies that all of these are installed and on the `PATH`,
emitting installation hints if anything is missing.

## The `git` variant

Generates the Go module under `target/<profile>/`, runs `go mod init` and
`go mod tidy` to populate `go.sum`, then commits the result and pushes it
to the configured remote with a `v<version>` tag.

### Example

```toml
# .buffy/golang.toml
[golang.git]
module = "github.com/example/tomato-go"
remote = "git@github.com:example/tomato-go.git"
branch = "main"
grpc = true
keep = ["README.md"]
```

### Fields

* [`module`](#the-module-field) --- The Go module path.
* [`remote`](#the-remote-field) --- Git remote URL.
* [`branch`](#the-branch-field) --- Branch to push to.
* [`grpc`](#the-grpc-field) --- Whether to generate gRPC service stubs.
* [`keep`](#the-keep-field) --- Files to preserve across publishes.

#### The `module` field

The Go module path, as it will appear in `go.mod` and as consumers will use
it in their `import` statements. Conventionally matches the host and path of
the `remote`.

```toml
module = "github.com/example/tomato-go"
```

This value is passed to `protoc-gen-go` as `--go_opt=module=...` so that the
generated package paths are rewritten correctly.

#### The `remote` field

The Git URL the generated module is pushed to. SSH URLs are recommended
because Buffy disables Git's terminal prompt; HTTPS URLs work only if
credentials are pre-cached or supplied via a credential helper.

```toml
remote = "git@github.com:example/tomato-go.git"
```

#### The `branch` field

The branch to push to. Buffy force-pushes the generated content to this
branch on every publish; the previous content is replaced (with the
exception of files listed in `keep`).

```toml
branch = "main"
```

#### The `grpc` field

When `true`, Buffy invokes `protoc-gen-go-grpc` in addition to
`protoc-gen-go`, generating service stubs alongside the message types. When
omitted or `false`, only message types are generated.

```toml
grpc = true
```

Default: `false`.

#### The `keep` field

A list of file paths (relative to the repository root) that Buffy fetches
from the remote before committing. Useful for human-maintained files like
`README.md` that should outlive the regenerated content.

```toml
keep = ["README.md", "docs/usage.md"]
```

If a listed file does not yet exist on the remote, Buffy logs a notice and
skips it instead of failing.

Default: `[]` (no files preserved).

### Example consumer usage

```sh
go get github.com/example/tomato-go@v0.1.0
```

```go
import (
    pb "github.com/example/tomato-go/greeter"
)
```
