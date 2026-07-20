# OCaml Tasks

For OCaml implementation tasks, read `docs/OCAML_GUIDE_FOR_AGENTS.md`.

Use only the guide section and linked official manual page relevant to the
current task. Do not guess unfamiliar OCaml APIs or import habits from Rust, F#,
Haskell, Base, or Core.

Use `pwsh.exe`, not `powershell.exe`. Run OCaml commands through the
project-local opam switch in `tethers-0.1/engine-ocaml`.

After a small OCaml change, compile immediately:

```powershell
pwsh -NoProfile -Command 'Push-Location .\tethers-0.1\engine-ocaml; opam exec -- dune build; exit $LASTEXITCODE'
```

Treat compiler diagnostics as evidence. If a fix attempt fails repeatedly, stop
and report the exact error rather than making speculative rewrites.
