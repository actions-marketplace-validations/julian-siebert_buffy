# The Manifest Format

The `Buffy.toml` file for each package is called its *manifest*. It is written
in the [TOML] format. It contains metadata that is needed to compile and publish
the protocol buffers to all supported language targets.

Every manifest file consists of the following sections:

* [`[package]`](#the-package-section) --- Defines a package.
  * [`name`](#the-name-field) --- The name of the package.
  * [`version`](#the-version-field) --- The version of the package.
  * [`description`](#the-description-field) --- A description of the package.
  * [`license`](#the-license-field) --- The package license.
  * [`homepage`](#the-homepage-field) --- URL of the package homepage.
  * [`authors`](#the-authors-field) --- The authors of the package.
* [`[source]`](#the-source-section) --- Configures where `.proto` files live.
  * [`path`](#the-path-field) --- Path to the directory containing `.proto` files.

Profile configuration (the per-language publishing targets in `.buffy/`) is
documented separately in [the profiles chapter](profiles.md).

## The `[package]` section

The first section in a `Buffy.toml` is `[package]`.

```toml
[package]
name = "tomato"   # the name of the package
version = "0.1.0" # the current version, obeying semver
```

All fields in this section are required. The metadata defined here is embedded
in every published artifact across all language targets (e.g., it ends up in
`Cargo.toml`, the Maven POM, `package.json`, and the AUTHORS/LICENSE files of
generated Go modules).

### The `name` field

The package name is an identifier used to refer to the package. It serves as
the default base name for language-specific artifacts (Cargo crate name, Maven
`artifactId`, npm package name, etc.). Profiles may override the name per
language.

The name must use only [alphanumeric] characters or `-` or `_`, and cannot be
empty.

* Only ASCII characters are allowed.
* Use a maximum of 32 characters of length.

```toml
[package]
name = "tomato"
```

[alphanumeric]: https://doc.rust-lang.org/std/primitive.char.html#method.is_alphanumeric

### The `version` field

The `version` field is formatted according to the [SemVer] specification:

Versions must have three numeric parts: the major version, the minor version,
and the patch version.

A pre-release part can be added after a dash such as `1.0.0-alpha`. The
pre-release part may be separated with periods to distinguish separate
components. Numeric components will use numeric comparison while everything
else will be compared lexicographically. For example, `1.0.0-alpha.11` is
higher than `1.0.0-alpha.4`.

A metadata part can be added after a plus, such as `1.0.0+21AF26D3`. This is
for informational purposes only and is generally ignored.

```toml
[package]
# ...
version = "0.1.0"
```

The version applies to all language targets when publishing. To override the
version for a single run (e.g., in CI where it is derived from a Git tag), use
the `--publish-version` flag:

```sh
buffy --publish --publish-version 1.2.3
```

[SemVer]: https://semver.org

### The `description` field

The description is a short blurb about the package. Registries that display it
(such as crates.io for the Rust target or npmjs.com for the JavaScript and
TypeScript targets) will show this text on the package page. This should be
plain text (not Markdown).

```toml
[package]
# ...
description = "Tomato protocol buffers for the salad service"
```

### The `license` field

The `license` field contains the SPDX expression of the software license that
the package is released under.

The value is interpreted as an [SPDX 2.3 license expression]. The name must be
a known license from the [SPDX license list]. SPDX expressions support `AND`
and `OR` operators to combine multiple licenses.

```toml
[package]
# ...
license = "MIT OR Apache-2.0"
```

Using `OR` indicates the user may choose either license. Using `AND` indicates
the user must comply with both licenses simultaneously. Some examples:

* `MIT`
* `MIT OR Apache-2.0`
* `LGPL-2.1-only AND MIT AND BSD-2-Clause`

When the manifest declares multiple licenses, Buffy generates one
`LICENSE-<id>` file per license plus an index `LICENSE` file describing the
combination. With a single license, a single `LICENSE` file is generated. The
full license text is embedded from the SPDX database.

Custom `LicenseRef-*` identifiers are not supported.

[SPDX 2.3 license expression]: https://spdx.github.io/spdx-spec/v2.3/SPDX-license-expressions/
[SPDX license list]: https://spdx.org/licenses/

### The `homepage` field

The `homepage` field should be a URL to a site that is the home page for your
package.

```toml
[package]
# ...
homepage = "https://github.com/example/tomato"
```

The homepage URL is surfaced in language-specific metadata fields where
applicable: `homepage` in `Cargo.toml`, `<url>` in the Maven POM, `homepage` in
`package.json`.

### The `authors` field

The `authors` field lists the people or organizations that are considered the
authors of the package. An optional email address may be included within
angled brackets at the end of each author entry.

```toml
[package]
# ...
authors = [
    "Jane Doe <jane@example.com>",
    "John Smith",
    "Acme Corp <opensource@acme.com>",
]
```

Each entry is parsed into a name and an optional email. Both are then forwarded
to language-specific metadata: `<developer>` entries in the Maven POM, the
`author` field in `package.json`, the `authors` array in `Cargo.toml`, and an
`AUTHORS` file in generated Go modules.

If an entry is malformed (e.g., empty, or containing brackets without an
email), Buffy reports a diagnostic pointing at the offending entry.

## The `[source]` section

The `[source]` section tells Buffy where to find the `.proto` files for code
generation.

```toml
[source]
path = "proto"
```

If the section is omitted, the default is used:

```toml
[source]
path = "src"
```

### The `path` field

The relative path (from `Buffy.toml`) to the directory containing `.proto`
files. Buffy walks this directory recursively and passes every `.proto` file
it finds to `protoc`.

The path is also used as `--proto_path` when invoking `protoc`, so imports
between `.proto` files should be relative to this root.

```toml
[source]
path = "proto"
```
