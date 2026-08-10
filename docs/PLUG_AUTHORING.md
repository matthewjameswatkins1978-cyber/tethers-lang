# Tethers Plug Authoring Manual

> A Plug gives Tethers a capability. A Tether decides when and why to use it.

> Deep Plug, narrow subject. Wide workflow, Tether.

This is the public manual for authoring a **Tethers Plug**. It documents only the
interfaces and behaviour that have actually been proven by the current Tethers
release (0.3, milestones P1–P3). It uses the real **Tethers PDF Tools** reference
Plug as its concrete example.

You should be able to read this manual top to bottom and then author, build,
pack, inspect, and conform a Plug of your own. The manual deliberately keeps
clarity and reproducibility ahead of comprehensiveness.

---

## 1. What a Plug is

A **Plug** is a bundle that gives Tethers a set of capabilities. A **Tether** is
a behavioural rule that decides when and why to use those capabilities. They are
different layers:

- A **Plug/provider** owns *application-specific meaning*: what a PDF file is,
  how to inspect it, what its metadata means.
- **Generic Tethers** owns *trust, packaging, scope evidence, supervision,
  dispatch, and receipts*: how a package is validated, how its executable is
  launched safely, how an action is dispatched, and how the outcome is recorded.

The mental boundary is:

> Deep Plug, narrow subject. Wide workflow, Tether.

A Plug is *not* itself a workflow. It does not decide whether a PDF should be
inspected in a particular situation. It simply makes "inspect this PDF" a
capability that Tethers can be told to use. A Tether ties that capability to an
event and to conditions.

Tethers is a deterministic planner. It proposes an ordered **Plan** of Actions;
it does not grant permission or execute Actions itself. The host application
enforces policy and executes. A Plug is the *capability provider* in that
arrangement.

---

## 2. Minimal authoring mental model

The public author journey is:

```text
author source
    ↓
plug pack
    ↓
.tetherplug package
    ↓
plug inspect
    ↓
plug conform
```

The author builds a **source tree** describing the Plug and its provider,
packages it into a `.tetherplug` file with `plug pack`, then inspects the result
and runs conformance.

One point cannot be overstated:

> **Conformance is not installation, and it does not create durable trust.**

`plug conform` proves a package behaves according to its declared contract. It
does not install the Plug, enable it, or make it permanently trusted. Those are
separate host-side lifecycle steps that happen later (if at all) and only through
the host's own policy.

---

## 3. Author source tree

The author supplies a source directory that describes the Plug and names its
provider executable and capability manifests.

**Tethers pack is not Cargo.** Your source code and build project may live
outside the temporary pack source. The pack source is a *curated* set of files,
not your whole repository.

For the PDF reference Plug the conceptual repository layout is:

```text
reference-plugs/pdf-tools/
├── provider-rust/                 # the provider's build project (source + Cargo)
├── author/                        # the curated pack source
│   ├── plug.json                  # the package descriptor (see section 4)
│   └── manifests/
│       └── pdf-inspect-v1.json    # a capability manifest (see section 5)
└── README.md
```

The compiled provider executable is copied into the *pack source* under a
`provider/` subdirectory before packing. The full conceptual pack source is
described in section 9.

Note: the reference provider happens to be written in Rust and built with Cargo,
but that is an implementation choice. Plug providers are **not** required to be
Rust, and `plug pack` has no knowledge of how a provider was built.

---

## 4. `plug.json`

`plug.json` is the package descriptor at the root of the pack source. It is the
file the author actually writes by hand. The following is based on the committed
PDF Tools `plug.json` (`reference-plugs/pdf-tools/author/plug.json`).

### Package identity and metadata

- `package_format_version` — the package format version, e.g. `"1"`.
- `package_id` — the stable package identifier, e.g. `"tethers.pdf-tools"`.
- `package_version` — the package version, e.g. `"1.0.0"`.
- `display_name` — a human-readable name, e.g. `"Tethers PDF Tools"`.
- `description` — a short description of what the Plug does.
- `publisher` — who authored/published the package.
- `licence` — the package licence, e.g. `"MIT"`.

### Socket and platform

- `socket_major` — the Plug socket major version (a compatibility gate), e.g.
  `1`.
- `protocol_bindings` — the protocol(s) the provider speaks. PDF Tools declares
  an MCP binding over stdio:
  `{"protocol":"MCP","version":"2025-11-25","transport":"stdio"}`.
- `platforms` — the platforms the provider supports, e.g.
  `[{"os":"windows","architecture":"x86_64"}]`.

### Provider

- `provider.provider_id` — the provider's trusted identity, e.g.
  `"tethers-pdf-provider"`.
- `provider.provider_version` — the provider's version.
- `provider.launch.path` — the provider executable path *relative to the
  `provider/` working directory*, e.g. `"provider/pdf_tools_provider.exe"`.
- `provider.launch.arguments` — static arguments, empty for PDF Tools.
- `provider.working_directory` — the working directory the provider is launched
  from, e.g. `"provider"`.
- `provider.capability_operation_namespace` — the namespace for the provider's
  operations, e.g. `"pdf"`.
- `provider.operational_scope_schema` — the JSON schema describing the
  operational scope the host will attach to this Plug (see section 7).

### Capabilities and payloads

- `capabilities` — a list of capability declarations. Each names:
  - `capability_name`, e.g. `"pdf.inspect"`;
  - `capability_version`, an integer, e.g. `1`;
  - `manifest_path`, the manifest file path relative to the pack source root,
    e.g. `"manifests/pdf-inspect-v1.json"`;
  - `provider_operation_name`, the operation the provider exposes for this
    capability, e.g. `"pdf_inspect"`.
- `payloads` — the list of files the author is shipping in the package. PDF
  Tools declares two: the capability manifest and the provider executable:

```json
"payloads": [
  {"path":"manifests/pdf-inspect-v1.json","role":"capability_manifest"},
  {"path":"provider/pdf_tools_provider.exe","role":"provider_executable"}
]
```

### Author declarations vs generated evidence

This is the most important distinction in the whole manual:

> **Authors write `payloads`. Packed packages contain a generated
> `payload_index`.**

You do **not** manually calculate or write:

- payload hashes;
- payload byte sizes;
- manifest digests;
- the semantic package digest.

Those values are **generated by Tethers** when the package is packed, based on
the exact bytes you ship. Hand-authoring them is both wrong and pointless: they
would not survive packing, because packing recomputes them from the real bytes.

---

## 5. Capability manifest

A **capability manifest** describes one capability. The PDF Tools manifest is at
`reference-plugs/pdf-tools/author/manifests/pdf-inspect-v1.json` and describes
`pdf.inspect@1`.

It contains:

- `capability_name` / `capability_version` — e.g. `"pdf.inspect"` / `1`.
- `title` / `description` — a human summary of the capability.
- `input_schema` — the JSON schema for the arguments a call must satisfy.
  PDF Tools declares `{"type":"object","properties":{"path":{"type":"string"}},
  "required":["path"],"additionalProperties":false}`.
- `output_schema` — the JSON schema for the result the provider returns.
- `effects` — declared external effects, e.g. `["data.read","metadata.read"]`.
- `permission_scope` — the host-side permission intent, e.g.
  `{"kind":"path_prefix","allowed_prefixes":["query/"]}`.
- `reversibility` / `determinism` — e.g. `"reversible"` / `"deterministic"`.
- `idempotency` — the idempotency contract, e.g. `{"mechanism":"none"}`.
- `confirmation_policy` — whether confirmation is required per call.
- `timeout_ms` / `retry_policy` — runtime execution controls.
- `provider` — the provider identity this capability binds to (must match the
  provider in `plug.json`), e.g. `"tethers-pdf-provider"`.
- `binding` — the protocol binding: `{"kind":"mcp","server_name":
  "tethers-pdf-provider","tool_name":"pdf_inspect","adapter":null}`.

The boundary again:

> **The author declares meaning. Tethers generates evidence.**

The manifest is something you write. The **manifest digest** is something Tethers
computes from the manifest bytes and records in the packed package. You do not
hand-edit or hand-write the digest.

---

## 6. Provider contract

The **provider** is the executable that actually performs the capability's work
when Tethers dispatches an action to it. The provider's responsibilities are:

- speak the declared protocol correctly (for MCP stdio providers, the MCP stdio
  protocol);
- only ever return results that conform to the capability's declared `output_schema`;
- only accept inputs that conform to the declared `input_schema`;
- perform no effect outside what the capability declares;
- respect the operational scope the host attaches (section 7).

For an **MCP stdio** provider, the wire contract is:

- `initialize` — negotiate protocol version and advertise server identity.
- `notifications/initialized` — acknowledge client initialization.
- `tools/list` — advertise the tools (operations) and their schemas.
- `tools/call` — perform the requested operation.

Transport discipline:

- **stdout is the protocol.** Every response is written as one JSON object per
  line on stdout. Do not write logs or diagnostics to stdout.
- **stderr is diagnostics.** Logs, configuration refusals, and errors the
  operator should see go to stderr.

Identity and schema discipline:

- the provider identity must match the trusted binding declared in `plug.json`
  and the manifest;
- the schemas the provider advertises in `tools/list` must match the reviewed
  manifest.

Providers must **not rely on host-family knowledge** — that is, a provider must
not assume it is running inside a particular host family. It should behave
according to its own declared protocol and scope.

This manual deliberately does not enumerate internal host Rust types. Those are
implementation details; the provider contract is defined at the protocol and
schema boundary above.

---

## 7. Operational Scope Evidence

**Operational Scope Evidence** is how a host constrains *where* and *how much*
a Plug may act, without the generic host needing to understand the
application-specific meaning.

The ownership split is the key idea:

> The **generic host** carries and validates the scope evidence.
> The **Plug/provider** interprets its own scope meaning.

Using PDF Tools as the example, the operational scope schema declares two
fields:

```text
query_root
max_bytes
```

The generic host does **not** know what a PDF path or PDF size means. It simply:

1. carries the scope schema (declared by the author in `plug.json`);
2. holds the concrete scope values as evidence tied to the Plug's identity and
   digest;
3. validates that scope evidence is well-formed and correctly bound.

The provider is the one that interprets those values as PDF-specific meaning:
`query_root` is the only directory it may read from, and `max_bytes` is the
largest file it will read.

### Environment contract

During provider execution the host supplies the operational scope and context
through environment variables:

```text
TETHERS_OPERATIONAL_SCOPE_JSON    the concrete scope object as JSON
TETHERS_OPERATIONAL_SCOPE_DIGEST  a digest of the scope (integrity anchor)
TETHERS_CONFORMANCE               "1" during host conformance
TEMP                              temp directory (used during conformance)
```

The provider reads these to configure itself. The PDF reference provider, for
example, reads `TETHERS_OPERATIONAL_SCOPE_JSON`, and during conformance uses
`TETHERS_CONFORMANCE` and `TEMP` to derive a safe temporary scope when no
installed scope is present.

The generic host never asserts what a PDF path *is*; it only guarantees that the
scope the provider sees is the scope that was validated and bound to this Plug.

---

## 8. Building the provider

The reference provider is built with Cargo because the reference happens to be a
Rust program. This is **not** a requirement for Plug providers in general; it is
simply the reference implementation's build step.

For the PDF Tools reference provider, from the repository root:

```powershell
cargo build --manifest-path reference-plugs/pdf-tools/provider-rust/Cargo.toml --locked
```

This produces the provider executable at:

```text
reference-plugs/pdf-tools/provider-rust/target/debug/pdf_tools_provider.exe
```

Any toolchain that produces a compliant provider executable is acceptable for a
Plug. What matters is the behaviour the provider exposes on the declared
protocol, not the language it is written in.

---

## 9. Assemble the temporary pack source

Before running `plug pack`, assemble a *temporary pack source* directory that
contains exactly the files the package should ship. For the PDF Tools Plug the
conceptual pack source is:

```text
plug.json
manifests/
    pdf-inspect-v1.json
provider/
    pdf_tools_provider.exe
```

The compiled provider executable is copied into the `provider/` subdirectory of
this temporary source. The build project and source code stay outside the pack
source.

This is why source code is not included accidentally: you curate the pack source,
so build intermediates, the Cargo project, and unrelated files never enter the
package unless you deliberately place them there.

---

## 10. `plug pack`

`plug pack` takes an author source directory and produces a deterministic
`.tetherplug` package.

The public CLI syntax is:

```text
plug pack --source <ABSOLUTE_DIRECTORY> --output <ABSOLUTE_FILE.tetherplug>
```

Example:

```powershell
plug pack --source path\to\pack-source --output path\to\pdf-tools.tetherplug
```

Both `--source` and `--output` are required. `--source` is the pack source
directory (section 9); `--output` is the resulting `.tetherplug` file.

At a high level, a successful `plug pack`:

- **validates** the author source (the descriptor, manifests, and payloads);
- **generates evidence** — payload hashes and sizes, manifest digests, and the
  semantic package digest — from the actual bytes being shipped;
- **writes a complete package** (the `.tetherplug` file) whose `plug.json` now
  carries the generated `payload_index` instead of the author's `payloads`;
- **does not mutate the author source**. Packing leaves your source files
  exactly as they were.

---

## 11. `plug inspect`

`plug inspect` treats a `.tetherplug` package as hostile, read-only data and
reports its contents without extracting, installing, or executing anything.

The public CLI syntax is:

```text
plug inspect --package <PATH>
```

Example:

```powershell
plug inspect --package path\to\pdf-tools.tetherplug
```

`--package` is required. Inspection reports:

- package identity (e.g. `package_id`);
- capability identity and its manifest digest;
- the semantic package digest;
- provider/package metadata (e.g. `provider_id`);
- the generated payload index.

Inspection **must not mutate the package**. It only reads and reports.

---

## 12. `plug conform`

`plug conform` runs the host conformance suite against a packaged Plug **without
installing it**.

The public CLI syntax is:

```text
plug conform --package <ABSOLUTE_FILE.tetherplug>
```

With explicit approval to execute:

```text
plug conform --package <ABSOLUTE_FILE.tetherplug> --allow-non-isolated-supervised-execution
```

### The default path: approval required

Without `--allow-non-isolated-supervised-execution`, conformance **refuses to
execute** the provider and requires approval. The command exits with an
`approval_required` status and an error code
`conformance_execution_approval_required`. In the PDF reference crucible the
denied conform exits with code `5`.

This refusal is fail-closed by design.

### The approved path: supervised, non-isolated execution

With `--allow-non-isolated-supervised-execution`, conformance performs its
execution. Be extremely clear about what this is and is not:

> **Public `plug conform` currently performs supervised but NON-isolated
> execution.**

- It **does NOT claim sandboxing** — the launch profile reports `isolated: false`
  and a limitation that the execution is "not isolated".
- It **does NOT install the Plug.**
- It **does NOT create durable trust or enablement.**

Supervision means the host launches the provider under its own control, constrains
the operational scope, and records the outcome. But the execution itself is not
run inside a sandbox. Treat a conform run accordingly: it is a supervised test of
a package against its declared contract, not a guarantee of isolation, and it
confers no lasting trust.

---

## 13. Complete PDF Tools walkthrough

This section ties the whole journey together using the real
`reference-plugs/pdf-tools/` reference Plug. It references the actual example
files rather than duplicating their full JSON content.

The reference files:

- `reference-plugs/pdf-tools/author/plug.json`
- `reference-plugs/pdf-tools/author/manifests/pdf-inspect-v1.json`
- `reference-plugs/pdf-tools/provider-rust/` (provider source)

### 1. Build the provider

```powershell
cargo build --manifest-path reference-plugs/pdf-tools/provider-rust/Cargo.toml --locked
```

The executable is produced at
`reference-plugs/pdf-tools/provider-rust/target/debug/pdf_tools_provider.exe`.

### 2. Assemble the pack source

Create a temporary pack source directory containing:

```text
plug.json
manifests/
    pdf-inspect-v1.json
provider/
    pdf_tools_provider.exe
```

Copy the committed `plug.json` and `manifests/pdf-inspect-v1.json`, and copy the
built `pdf_tools_provider.exe` into `provider/`.

### 3. Pack

```powershell
plug pack --source <pack-source> --output pdf-tools.tetherplug
```

### 4. Inspect

```powershell
plug inspect --package pdf-tools.tetherplug
```

### 5. Conform (default refusal)

```powershell
plug conform --package pdf-tools.tetherplug
```

This refuses execution and reports approval required.

### 6. Conform (approved supervised execution)

```powershell
plug conform --package pdf-tools.tetherplug --allow-non-isolated-supervised-execution
```

On success the conformance disposition is `passed` and the launch profile reports
`isolated: false` with a "not isolated" limitation.

The repository's own automated reproduction of this journey lives in the P3
crucible test `tethers-0.1/host-rust/tests/p3_pdf_reference_plug.rs`, and the
`just test-pdf-reference` recipe builds the provider and runs it.

---

## 14. Common mistakes

- **Putting undeclared source/build files inside the pack source.** The pack
  source is curated. Unrelated files and build output must not be added.
- **Writing `payload_index` instead of author `payloads`.** Authors write
  `payloads`. `payload_index` is generated by packing.
- **Manually adding generated digests.** Payload hashes and sizes, manifest
  digests, and the semantic package digest are generated. Hand-writing them is
  wrong.
- **Provider identity mismatch.** The identity in `plug.json`, the manifest
  `provider.identity`, and the identity the provider advertises must agree.
- **Tool/schema mismatch.** The operation name and schemas the provider
  advertises must match the capability declaration and manifest.
- **Missing provider executable.** The path named by
  `provider.launch.path` must exist inside the pack source's `provider/`.
- **Malformed operational scope.** The concrete scope values must satisfy the
  declared scope schema (for PDF Tools: a `query_root` directory and a bounded
  `max_bytes`).
- **Expecting conform to install or trust the Plug.** It does not.
- **Expecting conform to be sandboxed.** It is supervised but not isolated.
- **Using stdout for logs in an MCP stdio provider.** stdout is the protocol;
  diagnostics go to stderr.

---

## 15. Author checklist

Before you run `plug pack`, check:

1. `plug.json` declares correct package identity, version, and format version.
2. `plug.json` names the right provider identity and launch path.
3. The capability `manifest_path` points to a real, valid manifest.
4. `provider_operation_name` matches what the provider exposes.
5. The manifest `binding` tool name and server name match the provider.
6. `provider.identity` in the manifest matches `plug.json`.
7. The scope schema matches what the provider actually reads.
8. The input/output schemas in the manifest are the ones the provider implements.
9. Every file in `payloads` actually exists in the pack source.
10. No build output or unrelated files were added to the pack source.
11. The provider executable is present at the launch path.
12. The provider writes responses to stdout and diagnostics to stderr.
13. The pack source is a controlled temporary tree, not your whole repository.
14. You use `plug pack` to generate evidence — never hand-write digests.
15. You treat `plug conform` as supervised, non-isolated, non-installing, and
    non-trust-creating.
