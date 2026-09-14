// Narrow process boundary for the installed Tethers host.
//
// CallPermit does not reimplement Tethers and does not shell out through a
// command string.  This module passes an explicit argv vector to the current
// `tethers check` interface and parses its single JSON envelope.

import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

export interface TethersCliEnvelope {
  schema: string;
  command: string;
  status: string;
  exit_code: number;
  data?: Record<string, unknown>;
  error?: { code: string; message: string };
}

export interface TethersCheckOptions {
  config: string;
  engine: string;
  command?: string;
  cwd?: string;
}

export async function checkTethersRuntime(
  options: TethersCheckOptions,
): Promise<TethersCliEnvelope> {
  const command = options.command ?? "tethers";
  let stdout = "";
  try {
    ({ stdout } = await execFileAsync(command, [
      "check",
      "--config",
      options.config,
      "--engine",
      options.engine,
    ], {
      cwd: options.cwd,
      windowsHide: true,
      maxBuffer: 1024 * 1024,
      encoding: "utf8",
    }));
  } catch (error) {
    const failure = error as { stdout?: string };
    stdout = failure.stdout ?? "";
  }

  const line = stdout
    .split(/\r?\n/)
    .map((candidate) => candidate.trim())
    .find((candidate) => candidate.length > 0);
  if (!line) throw new Error("Tethers returned no JSON envelope");
  let envelope: TethersCliEnvelope;
  try {
    envelope = JSON.parse(line) as TethersCliEnvelope;
  } catch {
    throw new Error("Tethers returned malformed JSON");
  }
  if (envelope.schema !== "tethers.cli/1" || envelope.command !== "check") {
    throw new Error("Tethers returned an unexpected CLI envelope");
  }
  return envelope;
}
