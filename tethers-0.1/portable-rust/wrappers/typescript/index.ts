import { spawn } from "node:child_process";

export type Decision = "ALLOW" | "ASK" | "DENY";
export type Result = { decision: Decision; matched_rule?: string; rule?: string; reason?: string; error?: string };

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
      try { const result = JSON.parse(stdout); if (!["ALLOW", "ASK", "DENY"].includes(result.decision)) throw new Error("unknown Tethers decision"); finish(result); }
      catch (error) { finish({ decision: "DENY", error: `invalid Tethers response: ${error}` }); }
    });
    child.stdin.end(JSON.stringify(request));
  });
}
