"""Pure intent translators for the official Tethers plug surface."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any, Mapping


def request(action: str, *, actor: str, resource: str, context: Mapping[str, Any] | None = None,
            scope: Mapping[str, Any] | None = None) -> dict[str, Any]:
    result: dict[str, Any] = {"schema_version": "1", "actor": actor, "action": action,
                              "resource": resource, "context": dict(context or {})}
    if scope is not None:
        result["scope"] = dict(scope)
    return result


def git(operation: str, *, actor: str, repository: str, branch: str | None = None,
        remote: str | None = None) -> dict[str, Any]:
    actions = {"status":"git.status", "diff":"git.diff", "log":"git.log", "show":"git.show",
               "commit":"git.commit", "push":"git.push", "force_push":"git.force_push",
               "fetch":"git.fetch", "checkout":"git.checkout", "merge":"git.merge"}
    if operation not in actions:
        raise ValueError(f"unknown git operation: {operation}")
    context: dict[str, Any] = {"repository": repository}
    if branch is not None: context["branch"] = branch
    if remote is not None: context["remote"] = remote
    return request(actions[operation], actor=actor, resource=repository, context=context)


def filesystem(operation: str, *, actor: str, path: str, workspace_root: str,
               allowed_files: list[str] | None = None) -> dict[str, Any]:
    actions = {"read":"filesystem.read", "write":"filesystem.write", "create":"filesystem.create",
               "list":"filesystem.list", "stat":"filesystem.stat", "append":"filesystem.append",
               "rename":"filesystem.rename", "copy":"filesystem.copy", "move":"filesystem.move",
               "delete":"filesystem.delete", "mkdir":"filesystem.mkdir", "rmdir":"filesystem.rmdir"}
    if operation not in actions:
        raise ValueError(f"unknown filesystem operation: {operation}")
    root = Path(workspace_root).resolve(strict=False)
    candidate = Path(path)
    resolved = candidate.resolve(strict=False) if candidate.is_absolute() else (root / candidate).resolve(strict=False)
    try:
        relative = resolved.relative_to(root).as_posix()
        outside = False
    except ValueError:
        relative = path.replace("\\", "/")
        outside = True
    scope: dict[str, Any] = {"workspace_root": str(root), "allowed_actions": [actions[operation]]}
    if allowed_files is not None: scope["allowed_files"] = allowed_files
    context: dict[str, Any] = {"path": relative, "path_resolved": str(resolved), "path_resolution_failed": False}
    if outside: context["outside_workspace"] = True
    return request(actions[operation], actor=actor, resource=str(root), context=context, scope=scope)


def process(operation: str, *, actor: str, resource: str, command: str | None = None) -> dict[str, Any]:
    actions = {"execute":"process.execute", "shell":"process.execute_shell", "spawn":"process.spawn",
               "background":"process.background", "kill":"process.kill"}
    if operation not in actions:
        raise ValueError(f"unknown process operation: {operation}")
    context: dict[str, Any] = {"command_declared": command is not None}
    if command is not None: context["command"] = command
    return request(actions[operation], actor=actor, resource=resource, context=context)


def network(operation: str, *, actor: str, url: str) -> dict[str, Any]:
    actions = {"get":"network.http_get", "head":"network.http_head", "post":"network.http_post",
               "put":"network.http_put", "patch":"network.http_patch", "delete":"network.http_delete",
               "download":"network.download", "upload":"network.upload", "resolve":"network.resolve",
               "connect":"network.connect", "websocket":"network.websocket"}
    if operation not in actions:
        raise ValueError(f"unknown network operation: {operation}")
    return request(actions[operation], actor=actor, resource=url, context={"url": url})


def secret(operation: str, *, actor: str, name: str) -> dict[str, Any]:
    actions = {"exists":"secret.exists", "use":"secret.use", "read":"secret.read",
               "write":"secret.write", "export":"secret.export", "expose":"secret.expose"}
    if operation not in actions:
        raise ValueError(f"unknown secret operation: {operation}")
    return request(actions[operation], actor=actor, resource=f"secret:{name}", context={"secret_name": name})


def container(operation: str, *, actor: str, image: str) -> dict[str, Any]:
    actions = {"build":"container.build", "run":"container.run", "exec":"container.exec",
               "stop":"container.stop", "remove":"container.remove", "pull":"container.image.pull",
               "push":"container.image.push", "network":"container.network", "mount":"container.mount",
               "privileged":"container.privileged"}
    if operation not in actions:
        raise ValueError(f"unknown container operation: {operation}")
    return request(actions[operation], actor=actor, resource=f"container:{image}", context={"image": image})


def tool(name: str, *, actor: str, arguments: Mapping[str, Any] | None = None) -> dict[str, Any]:
    return request("tool.call", actor=actor, resource=f"tool:{name}", context={"tool": name, "arguments_present": bool(arguments)})


def mcp_tool(name: str, *, actor: str, definition: Mapping[str, Any], arguments: Mapping[str, Any] | None = None) -> dict[str, Any]:
    canonical = json.dumps(definition, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    fingerprint = hashlib.sha256(canonical).hexdigest()
    return request("mcp.tool_call", actor=actor, resource=f"mcp:{name}", context={"tool": name, "tool_definition_sha256": fingerprint, "arguments_present": bool(arguments)})


def database(operation: str, *, actor: str, resource: str) -> dict[str, Any]:
    actions = {"connect":"database.connect", "read":"database.read", "write":"database.write",
               "insert":"database.insert", "update":"database.update", "delete":"database.delete",
               "drop":"database.drop", "schema_read":"database.schema_read"}
    if operation not in actions:
        raise ValueError(f"unknown database operation: {operation}")
    return request(actions[operation], actor=actor, resource=resource, context={"operation": operation})


def message(operation: str, *, actor: str, resource: str, email: bool = False) -> dict[str, Any]:
    prefix = "email" if email else "message"
    action = f"{prefix}.{operation}"
    allowed = {"message.draft", "message.send", "message.read", "message.delete",
               "email.draft", "email.send", "email.read", "email.forward", "email.delete"}
    if action not in allowed:
        raise ValueError(f"unknown messaging operation: {operation}")
    return request(action, actor=actor, resource=resource, context={"operation": operation})


def deploy(operation: str, *, actor: str, resource: str) -> dict[str, Any]:
    action = f"deploy.{operation}"
    allowed = {"deploy.inspect", "deploy.build", "deploy.preview", "deploy.staging", "deploy.production", "deploy.rollback", "deploy.destroy"}
    if action not in allowed:
        raise ValueError(f"unknown deployment operation: {operation}")
    return request(action, actor=actor, resource=resource, context={"operation": operation})
