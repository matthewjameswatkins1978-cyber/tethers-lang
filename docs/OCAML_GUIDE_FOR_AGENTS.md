# OCaml Guide For Tethers Agents

## A. Purpose And Version Contract

This guide is for AI coding agents making OCaml changes in Tethers. It is a
compact project guide, not a replacement for the official OCaml manual.

Tethers currently targets OCaml `5.5.0` in its project-local opam switch. Do
not assume syntax or APIs from Rust, F#, Haskell, Standard ML, Base, Core, or
another ML-family environment apply to OCaml.

When behaviour is version-specific or unfamiliar, check the linked official
documentation. The compiler and Tethers contract tests are the final authority.

## B. Exact Environment

Verified on 2026-07-20:

- OCaml compiler: `5.5.0`
- opam: `2.5.2`
- Dune: `3.24.0`
- Yojson: `2.2.2`
- PowerShell automation shell: PowerShell 7.6.3, `pwsh.exe`
- Local switch path: `D:\The Next Thing\Tethers Lang\tethers-0.1\engine-ocaml`

The OCaml switch is path-bound and lives under:

```text
tethers-0.1/engine-ocaml/_opam/
```

Do not move or rename `tethers-0.1/engine-ocaml` while this switch is in use.
Use `opam exec` from inside `tethers-0.1/engine-ocaml` so commands run against
the local switch.

The package constraints in `tethers-0.1/engine-ocaml/tethers_engine.opam` are
`ocaml >= 5.1.0 and < 6.0.0`, `dune >= 3.10`, and
`yojson >= 2.0.0 and < 3.0.0`. The installed local switch is more specific:
OCaml `5.5.0`, Dune `3.24.0`, Yojson `2.2.2`.

## C. Exact Commands

Use `pwsh`, never `powershell.exe`, for project automation.

Check the active toolchain and local switch:

```powershell
pwsh -NoProfile -Command '$PSVersionTable.PSVersion'
pwsh -NoProfile -Command 'Push-Location .\tethers-0.1\engine-ocaml; opam switch show; opam exec -- ocamlc -version; opam exec -- dune --version; opam list --installed --short --columns=name,version | Select-String -Pattern "^(ocaml|ocaml-base-compiler|dune|yojson)\s"; Pop-Location'
```

Build and verify:

```powershell
pwsh -NoProfile -Command 'Push-Location .\tethers-0.1\engine-ocaml; opam exec -- dune build; exit $LASTEXITCODE'
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\check-fixtures.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\test-engine.ps1
pwsh -NoProfile -ExecutionPolicy Bypass -File .\tethers-0.1\scripts\demo.ps1
```

For Rust host unit tests:

```powershell
pwsh -NoProfile -Command 'Push-Location .\tethers-0.1\host-rust; cargo test; exit $LASTEXITCODE'
```

Compile immediately after a small OCaml edit. Treat compiler diagnostics as
evidence, not as an invitation to speculative rewrites.

## D. OCaml Subset Used By Tethers

Tethers currently uses a small, ordinary OCaml subset.

Algebraic data types model closed choices:

```ocaml
type operator = Is | Contains | Greater_than | Greater_than_or_equal
```

Pattern matching makes those choices explicit:

```ocaml
match (condition.operator, actual, condition.expected) with
| Is, String_value left, String_value right -> left = right
| Greater_than, Int_value left, Int_value right -> left > right
| _ -> fail "type_error" ("Invalid operands in condition: " ^ condition.source)
```

Records model named Tethers concepts:

```ocaml
type condition = {
  fact : string;
  operator : operator;
  expected : value;
  source : string;
}
```

Lists are immutable. Code builds new lists with `::`, `@`, `List.rev`,
`List.map`, `List.filter`, `List.fold_left`, and related standard-library
functions. Keep ordering deterministic because Trail sequence and Action order
are semantic.

Recursion is used for parsing and evaluation loops:

```ocaml
let rec check_conditions sequence trail = function
  | [] -> Conditions_matched (sequence, trail)
  | condition :: rest -> ...
```

Modules are ordinary compilation units. The project has no `.mli` interfaces
yet, so exported names come from the `.ml` files themselves.

Exceptions currently carry structured engine error codes:

```ocaml
exception Tethers_error of string * string
let fail code message = raise (Tethers_error (code, message))
```

Keep existing error codes and messages stable unless a fixture intentionally
changes them.

Options are handled explicitly with `Some` and `None`, for example
`List.assoc_opt`, `String.index_opt`, and `int_of_string_opt`.

JSON conversion is explicit. Tethers converts between `Yojson.Safe.t` and the
declared 0.1 value subset rather than passing arbitrary JSON deeper into the
language evaluator.

## E. Module Structure

Current direct dependency graph:

```text
main
  -> Tethers_protocol
  -> Tether_parser

Tethers_protocol
  -> Tether_parser
```

The graph is acyclic. Do not introduce circular dependencies.

One `.ml` file becomes one capitalised module. For example:

- `main.ml` builds the executable entry point.
- `tether_parser.ml` is module `Tether_parser`.
- `tethers_protocol.ml` is module `Tethers_protocol`.

Lower-case filenames produce capitalised module names because OCaml derives a
compilation-unit name from the file name. Other modules refer to that unit by
the capitalised form.

This project uses `open Tethers_protocol` and `open Tether_parser` in `main.ml`
to keep the small reference engine readable. Use `open` sparingly. If a future
module becomes large or names collide, prefer qualified names.

Parser or protocol extraction must preserve behaviour and be protected by the
fixture suite. After moving code, run `opam exec -- dune build` and
`test-engine.ps1`.

## F. Yojson Usage

Tethers currently uses `Yojson.Safe.t` as the JSON tree type.

Value forms currently constructed or matched:

- `` `Assoc`` for JSON objects.
- `` `List`` for JSON arrays.
- `` `String`` for strings.
- `` `Int`` for integers.
- `` `Bool`` for booleans.
- `` `Null`` for null plan values.

APIs currently used:

- `Yojson.Safe.from_string` to parse one request line.
- `Yojson.Safe.to_string` to print one response line.
- `Yojson.Safe.Util.member` to read object fields.

Yojson supports more representations than Tethers 0.1 accepts, including floats
and non-standard extensions. Tethers 0.1 intentionally accepts only its declared
value subset: strings, integers, booleans, and `anchor.*` references in Tether
source.

## G. Project OCaml Style

- Favour small explicit algebraic data types.
- Prefer clear pattern matching over clever control flow.
- Use immutable data unless mutation has a concrete justification.
- Avoid combinator-heavy code that obscures Tether semantics.
- Avoid unnecessary abstraction.
- Preserve exact error codes and messages during refactors.
- Keep deterministic ordering for Conditions, Actions, Effects, and Trail.
- Avoid hidden I/O in evaluation code.
- Do not add Base, Core, or another alternative standard library.
- Do not use new OCaml 5.5 features merely because they exist.
- Use the simplest stable OCaml feature that clearly expresses the behaviour.

## H. Common AI Mistakes

- Inventing functions from Rust, F#, Haskell, Base, or Core that do not exist in
  the OCaml standard library.
- Confusing structural equality `=` with physical equality `==`.
- Silently coercing integers, strings, and booleans.
- Treating JSON object field order as semantically meaningful.
- Changing exception messages during mechanical refactoring.
- Losing significant indentation inside embedded Tether source strings.
- Turning deterministic evaluation into filesystem, clock, environment, or
  network access.
- Using a wildcard case that hides newly added variants.
- Claiming an unrun test passed.
- Reformatting an entire file for a small change.

## I. When To Consult The Official Manual

| Task | Documentation to consult |
| --- | --- |
| Module/interface change | OCaml modules and compilation units |
| New variant or pattern | Variants and pattern matching |
| Exception/result change | Error handling and exceptions |
| Dune stanza change | Current Dune reference |
| Dependency change | opam package metadata |
| JSON representation change | Yojson Safe API |
| New OCaml 5.5 feature | Exact OCaml 5.5 manual section |

If the task is unfamiliar or version-specific, consult the linked official
source first, make the smallest project-shaped change, compile, then run the
relevant contract tests.

## J. Sources

- OCaml 5.5 manual and docs: <https://ocaml.org/manual/5.5/>,
  <https://ocaml.org/docs>
- OCaml language pages: <https://ocaml.org/manual/5.5/compunit.html>,
  <https://ocaml.org/manual/5.5/coreexamples.html>,
  <https://ocaml.org/manual/5.5/patterns.html>,
  <https://ocaml.org/manual/5.5/typedecl.html>
- OCaml standard library APIs: <https://ocaml.org/manual/5.5/api/List.html>,
  <https://ocaml.org/manual/5.5/api/String.html>,
  <https://ocaml.org/manual/5.5/api/Option.html>
- Dune reference: <https://dune.readthedocs.io/en/stable/reference/dune/index.html>
- opam manual: <https://opam.ocaml.org/doc/Manual.html>
- Yojson Safe API: <https://ocaml-community.github.io/yojson/yojson/Yojson/Safe/index.html>
- Yojson repository: <https://github.com/ocaml-community/yojson>
