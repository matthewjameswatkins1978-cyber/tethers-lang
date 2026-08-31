package tethers

import "testing"

func TestUnknownDecisionFailsClosed(t *testing.T) {
	if got := decode([]byte(`{"decision":"WHAT"}`)); got.Decision != Deny { t.Fatalf("got %q", got.Decision) }
}

func TestMalformedResponseFailsClosed(t *testing.T) {
	if got := decode([]byte("not json")); got.Decision != Deny { t.Fatalf("got %q", got.Decision) }
}

func TestSchemaMismatchFailsClosed(t *testing.T) {
	if got := decode([]byte(`{"schema_version":"2","decision":"ALLOW"}`)); got.Decision != Deny { t.Fatalf("got %q", got.Decision) }
}
