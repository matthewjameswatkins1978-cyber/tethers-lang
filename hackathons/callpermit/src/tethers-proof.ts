// Explicit, read-only Tethers runtime proof command.
// It performs `tethers check` only; it never evaluates a call or makes a
// network request.  Paths are supplied by the caller so no repository files
// or hidden configuration are guessed.

import { checkTethersRuntime } from "./tethers-cli.js";

const config = process.env.TETHERS_CONFIG;
const engine = process.env.TETHERS_ENGINE;
if (!config || !engine) {
  console.error("Set TETHERS_CONFIG and TETHERS_ENGINE to run the Tethers check proof.");
  process.exitCode = 2;
} else {
  try {
    const envelope = await checkTethersRuntime({ config, engine });
    console.log(JSON.stringify(envelope, null, 2));
    process.exitCode = envelope.exit_code;
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
