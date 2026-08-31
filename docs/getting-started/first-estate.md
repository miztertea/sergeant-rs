# Create your first estate

An estate is the exact directory containing `sergeant.toml`. Sergeant never searches parent directories for it.

```sh
mkdir -p ~/estates/example
cd ~/estates/example
sgt init
sgt repo add app --origin https://github.com/example/app.git
sgt doctor
sgt repo list
```

`sgt init` writes the embedded AgentOS distribution: `AGENTS.md`, a `CLAUDE.md -> AGENTS.md` symlink, skills, workflows, and the estate manifest. A repository mount is always `repos/<name>`; it is a clean base checkout, not a worker checkout.

Use `sgt -C ~/estates/example <command>` when you want to address the exact root without changing directories. Add a group when several repositories form a frequent scope:

```sh
sgt group add product app
sgt group list
```

Run `sgt doctor` after changing topology. It names each failed check and its remedy.
