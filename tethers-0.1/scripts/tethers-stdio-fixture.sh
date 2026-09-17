#!/bin/sh

# Keep the Linux entry point POSIX, but use Python for the JSON protocol so the
# fixture has the same modes and request semantics as the Windows fixture.
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec python3 "$script_dir/tethers-stdio-fixture.py" "$@"
