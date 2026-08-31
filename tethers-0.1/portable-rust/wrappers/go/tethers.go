package tethers

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os/exec"
	"time"
)

type Decision string
const ( Allow Decision = "ALLOW"; Ask Decision = "ASK"; Deny Decision = "DENY" )
type Result struct { Decision Decision `json:"decision"`; Rule string `json:"matched_rule"`; Reason string `json:"reason"`; Error string `json:"error"` }

func Evaluate(binary string, request []byte, timeout time.Duration) Result {
	ctx, cancel := context.WithTimeout(context.Background(), timeout); defer cancel()
	cmd := exec.CommandContext(ctx, binary, "evaluate"); cmd.Stdin = bytes.NewReader(request)
	out, err := cmd.Output()
	if ctx.Err() != nil { return Result{Decision: Deny, Error: "Tethers evaluation timed out"} }
	if err != nil { return Result{Decision: Deny, Error: fmt.Sprintf("Tethers failed: %v", err)} }
	return decode(out)
}

func decode(out []byte) Result {
	var result Result
	if err := json.Unmarshal(out, &result); err != nil { return Result{Decision: Deny, Error: "invalid Tethers response"} }
	if result.Decision != Allow && result.Decision != Ask && result.Decision != Deny { return Result{Decision: Deny, Error: "unknown Tethers decision"} }
	return result
}
