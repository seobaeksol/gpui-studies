# GPUI Studies

Small, independently runnable projects for studying [GPUI](https://www.gpui.rs/).

## Projects

| Package | Description | Run |
| --- | --- | --- |
| `gpui-playground` | Basic GPUI components and interaction experiments | `cargo run -p gpui-playground` |
| `system-monitor` | System metrics dashboard | `cargo run -p system-monitor` |

## Development

Run the playground, which is the default workspace member:

```powershell
cargo run
```

Check every project in the workspace:

```powershell
cargo check --workspace
```

Useful references:

- [GPUI documentation](https://github.com/zed-industries/zed/tree/main/crates/gpui/docs)
- [GPUI examples](https://github.com/zed-industries/zed/tree/main/crates/gpui/examples)
