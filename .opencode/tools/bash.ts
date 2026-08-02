// OpenCode custom bash tool — execution-environment enforcement gate.
//
// This overrides the built-in bash tool. It accepts only:
//
//   tethers-run <approved-command-id>
//
// and delegates to `tethers-env run` through the frozen contract's permit()
// boundary and SupervisedChild launch. All arbitrary shell commands,
// pipelines, redirections, executable paths, and shell metacharacters
// are refused.

export default async ({ directory }: { directory: string }) => {
  return {
    "tool.execute.before": async (input: any, output: any) => {
      if (input.tool !== "bash") return;

      const cmd = output?.args?.command;
      if (cmd === undefined || cmd === null) {
        throw new Error(
          "no shell command supplied. Only 'tethers-run <approved-command-id>' is permitted."
        );
      }

      const cmdStr = String(cmd).trim();
      if (cmdStr === "") {
        throw new Error("empty command — use tethers-run <approved-command-id>");
      }

      const match = cmdStr.match(/^tethers-run\s+([a-zA-Z][a-zA-Z0-9_-]{0,63})$/);
      if (!match) {
        throw new Error(
          "arbitrary shell commands are not permitted by this worktree.\n" +
            "Use: tethers-run <approved-command-id>\n" +
            "Examples: tethers-run git-status, tethers-run rust-check-offline, tethers-run rust-tests"
        );
      }

      const commandId = match[1];
      const envExe = `${directory}\\tethers-0.1\\host-rust\\target\\debug\\tethers-env.exe`;
      const contractPath = `${directory}\\.tethers\\execution\\contract.json`;

      output.args.command =
        `"${envExe}" run --contract "${contractPath}" --command-id "${commandId}"`;
    },
  };
};
