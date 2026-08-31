import { spawn } from "node:child_process";

export type Decision = "ALLOW" | "ASK" | "DENY";
export type Result = { schema_version?: string; decision: Decision; matched_rule?: string; rule?: string; reason?: string; error?: string };

export function parseResponse(stdout: string): Result {
  try {
    const result = JSON.parse(stdout);
    if (result.schema_version !== "1") throw new Error("Tethers response schema mismatch");
    if (!["ALLOW", "ASK", "DENY"].includes(result.decision)) throw new Error("unknown Tethers decision");
    return result;
  } catch (error) {
    return { decision: "DENY", error: `invalid Tethers response: ${error}` };
  }
}

export function evaluate(binary: string, request: unknown, timeoutMs = 5000): Promise<Result> {
  return new Promise((resolve) => {
    let stdout = "";
    let settled = false;
    const child = spawn(binary, ["evaluate"], { stdio: ["pipe", "pipe", "ignore"] });
    const finish = (result: Result) => { if (!settled) { settled = true; clearTimeout(timer); resolve(result); } };
    const timer = setTimeout(() => { child.kill(); finish({ decision: "DENY", error: "Tethers evaluation timed out" }); }, timeoutMs);
    child.stdout.on("data", (chunk) => { stdout += chunk.toString(); });
    child.on("error", (error) => finish({ decision: "DENY", error: `cannot start Tethers: ${error.message}` }));
    child.on("close", (code) => {
      if (code !== 0) return finish({ decision: "DENY", error: `Tethers exited with ${code}` });
      finish(parseResponse(stdout));
    });
    child.stdin.end(JSON.stringify(request));
  });
}
