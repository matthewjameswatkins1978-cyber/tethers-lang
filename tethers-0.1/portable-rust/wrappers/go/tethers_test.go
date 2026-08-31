package tethers

import "testing"

func TestUnknownDecisionFailsClosed(t *testing.T) {
	if got := decode([]byte(`{"decision":"WHAT"}`)); got.Decision != Deny { t.Fatalf("got %q", got.Decision) }
}

func TestMalformedResponseFailsClosed(t *testing.T) {
	if got := decode([]byte("not json")); got.Decision != Deny { t.Fatalf("got %q", got.Decision) }
}
