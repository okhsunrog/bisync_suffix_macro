# bisync_suffix_macro

A procedural macro to conditionally append suffixes to method names in `.await` expressions, enabling dual support for asynchronous and blocking code paths. This macro is designed to work seamlessly with the `bisync` crate, allowing functions to support both async and blocking execution models based on feature flags.

## Description

The `bisync_suffix_macro` crate provides the `suffix` macro, which transforms method names in `.await` expressions by appending a specified suffix when the `async` feature is enabled. When the `blocking` feature is enabled (and `async` is not), the original method name is preserved. This macro is particularly useful in crates that need to provide both async and blocking APIs from a single codebase, such as the `axp192-dd` crate.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
bisync_suffix_macro = "0.1.0"
```

## Usage

The `suffix` macro is typically used within functions annotated with `#[bisync]` from the `bisync` crate. It takes two arguments: a string literal suffix (e.g., `"_async"`) and an expression containing `.await` calls.

When the `async` feature is enabled, the macro appends the suffix to method names in `.await` expressions. When the `blocking` feature is enabled, the original method name is used, and `#[bisync]` removes the `.await` to adapt the function for blocking execution.

For detailed usage and examples, refer to the [crate documentation](https://docs.rs/bisync_suffix_macro).

## License

This crate is dual-licensed under the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
