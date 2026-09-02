# Use Sergeant directly

Captain is optional. The direct path is:

```sh
mkdir estate && cd estate
sgt init
sgt repo add app --origin <git-url>
sgt run "accepted intent" --repo app --workflow implement-change --backend claude
sgt watch <work-id> --follow
sgt work show <work-id>
```

Use `--group <name>` for a declared group or `--all` for every repository; `--repo` and `--group` combine (union, dedup), but `--all` is mutually exclusive with both. Global `--json` switches supported surfaces to machine output. Global `-C <root>` addresses an estate explicitly.

Read `sgt --help` and `sgt <command> --help` for the exact grammar; see the [CLI reference](../reference/cli.md) for behavior and cross-links.
