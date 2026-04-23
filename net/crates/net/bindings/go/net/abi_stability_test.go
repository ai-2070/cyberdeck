// ABI-stability regression tests for the Go binding's error
// surface. These pin the wire-format SDK callers match on:
//
//   1. The `daemon:` / `migration:` / `traversal:` / `channel:`
//      message prefixes produced by `*DaemonError`,
//      `*DuplicateKindError`, `*MigrationError`, and the
//      package-level `Err*` sentinels.
//   2. The stable `MigrationErrorKind` string vocabulary — a
//      binding release that renames one of these breaks every
//      caller's switch statement, so each kind gets a hard-
//      coded equality assertion.
//   3. `parseMigrationError` correctly lifts every valid kind
//      out of a synthesized `daemon: migration: <kind>[: detail]`
//      message, and demotes unknown kinds to
//      `MigrationErrKindUnknown` (forward-compat contract).
//
// These tests don't touch any cgo handle and require no live
// mesh — they operate on the Go-side type surface only, so a
// rename in `migration.go` flags here before it propagates into
// an SDK release.
//
// Corresponds to TEST_COVERAGE_PLAN §P3-15.
//
// The FFI round-trip test that actually exercises the Go↔C
// uint64 boundary lives in `abi_stability_cgo_test.go`, gated
// on `//go:build test_helpers` — cgo directives aren't allowed
// inside `_test.go` files, so the helper lives in a paired
// non-test file with the same build tag.

package net

import (
	"errors"
	"strings"
	"testing"
)

// TestABIStabilityDaemonErrorPrefix pins that *DaemonError always
// serializes with exactly the "daemon: " prefix. Any other prefix
// would break `classifyError`-style helpers on the Node side and
// the `migration_error_kind` parser on the Python side, since
// they share this envelope convention.
func TestABIStabilityDaemonErrorPrefix(t *testing.T) {
	cases := []struct {
		name string
		msg  string
		want string
	}{
		{"empty message", "", "daemon: "},
		{"simple detail", "factory not found", "daemon: factory not found"},
		{"nested prefix", "migration: not-ready", "daemon: migration: not-ready"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			e := &DaemonError{Message: tc.msg}
			if got := e.Error(); got != tc.want {
				t.Fatalf("DaemonError.Error() = %q, want %q — \"daemon: \" prefix is a wire contract", got, tc.want)
			}
		})
	}
}

// TestABIStabilityDuplicateKindErrorFormat pins the exact format
// string `DuplicateKindError` emits — callers who regex on the
// kind name (for structured dispatch) depend on it staying in
// single-quotes.
func TestABIStabilityDuplicateKindErrorFormat(t *testing.T) {
	e := &DuplicateKindError{Kind: "counter"}
	const want = "daemon: factory for kind 'counter' is already registered"
	if got := e.Error(); got != want {
		t.Fatalf("DuplicateKindError.Error() = %q, want %q", got, want)
	}
}

// TestABIStabilityTraversalSentinels pins every `ErrTraversal*`
// sentinel's `.Error()` value. `errors.Is` relies on pointer
// equality, but callers that log+match on substrings (common in
// operator tooling) need the string values stable too.
func TestABIStabilityTraversalSentinels(t *testing.T) {
	cases := []struct {
		name string
		err  error
		want string
	}{
		{"ReflexTimeout", ErrTraversalReflexTimeout, "traversal: reflex-timeout"},
		{"PeerNotReachable", ErrTraversalPeerNotReachable, "traversal: peer-not-reachable"},
		{"Transport", ErrTraversalTransport, "traversal: transport"},
		{"RendezvousNoRelay", ErrTraversalRendezvousNoRelay, "traversal: rendezvous-no-relay"},
		{"RendezvousRejected", ErrTraversalRendezvousRejected, "traversal: rendezvous-rejected"},
		{"PunchFailed", ErrTraversalPunchFailed, "traversal: punch-failed"},
		{"PortMapUnavailable", ErrTraversalPortMapUnavailable, "traversal: port-map-unavailable"},
		{"Unsupported", ErrTraversalUnsupported, "traversal: unsupported"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.err == nil {
				t.Fatalf("sentinel is nil — removed without replacement?")
			}
			if got := tc.err.Error(); got != tc.want {
				t.Fatalf("sentinel %q = %q, want %q", tc.name, got, tc.want)
			}
			if !strings.HasPrefix(tc.err.Error(), "traversal: ") {
				t.Fatalf("sentinel %q lost the \"traversal: \" prefix", tc.name)
			}
		})
	}
}

// TestABIStabilityChannelAuthSentinel pins the channel error.
func TestABIStabilityChannelAuthSentinel(t *testing.T) {
	if ErrChannelAuth == nil {
		t.Fatal("ErrChannelAuth sentinel is nil")
	}
	const want = "channel: unauthorized"
	if got := ErrChannelAuth.Error(); got != want {
		t.Fatalf("ErrChannelAuth = %q, want %q", got, want)
	}
}

// TestABIStabilityMigrationErrorKinds pins every
// `MigrationErrorKind` string literal. The set + values are the
// cross-binding vocabulary: renaming "not-ready" to "notReady"
// breaks Go, Python, *and* Node callers simultaneously, so the
// assertion is deliberately strict (not lower/upper-case
// insensitive).
func TestABIStabilityMigrationErrorKinds(t *testing.T) {
	cases := []struct {
		kind MigrationErrorKind
		want string
	}{
		{MigrationErrKindNotReady, "not-ready"},
		{MigrationErrKindFactoryNotFound, "factory-not-found"},
		{MigrationErrKindComputeNotSupported, "compute-not-supported"},
		{MigrationErrKindStateFailed, "state-failed"},
		{MigrationErrKindAlreadyMigrating, "already-migrating"},
		{MigrationErrKindIdentityTransportFailed, "identity-transport-failed"},
		{MigrationErrKindNotReadyTimeout, "not-ready-timeout"},
		{MigrationErrKindDaemonNotFound, "daemon-not-found"},
		{MigrationErrKindTargetUnavailable, "target-unavailable"},
		{MigrationErrKindWrongPhase, "wrong-phase"},
		{MigrationErrKindSnapshotTooLarge, "snapshot-too-large"},
		{MigrationErrKindUnknown, "unknown"},
	}
	for _, tc := range cases {
		if string(tc.kind) != tc.want {
			t.Errorf("MigrationErrorKind %q bound to %q, want %q", tc.want, string(tc.kind), tc.want)
		}
	}
}

// TestABIStabilityParseMigrationErrorRoundTrip pins the parser:
// for every known kind, a synthesized `migration: <kind>[: detail]`
// message on a `DaemonError` lifts into a typed `*MigrationError`
// carrying exactly that kind. Unknown kinds demote to
// `MigrationErrKindUnknown` so SDK callers never see a nil or a
// surprise string.
func TestABIStabilityParseMigrationErrorRoundTrip(t *testing.T) {
	knownKinds := []MigrationErrorKind{
		MigrationErrKindNotReady,
		MigrationErrKindFactoryNotFound,
		MigrationErrKindComputeNotSupported,
		MigrationErrKindStateFailed,
		MigrationErrKindAlreadyMigrating,
		MigrationErrKindIdentityTransportFailed,
		MigrationErrKindNotReadyTimeout,
		MigrationErrKindDaemonNotFound,
		MigrationErrKindTargetUnavailable,
		MigrationErrKindWrongPhase,
		MigrationErrKindSnapshotTooLarge,
	}
	for _, k := range knownKinds {
		t.Run(string(k), func(t *testing.T) {
			// Tag-only form.
			d := &DaemonError{Message: "migration: " + string(k)}
			me := parseMigrationError(d)
			if me == nil {
				t.Fatalf("parseMigrationError returned nil for tag-only kind %q", k)
			}
			if me.Kind != k {
				t.Errorf("kind = %q, want %q", me.Kind, k)
			}
			if me.Detail != "" {
				t.Errorf("detail = %q, want empty for tag-only form", me.Detail)
			}

			// Tag + detail form.
			d2 := &DaemonError{Message: "migration: " + string(k) + ": boom"}
			me2 := parseMigrationError(d2)
			if me2 == nil {
				t.Fatalf("parseMigrationError returned nil for tag+detail kind %q", k)
			}
			if me2.Kind != k {
				t.Errorf("kind with detail = %q, want %q", me2.Kind, k)
			}
			if me2.Detail != "boom" {
				t.Errorf("detail = %q, want %q", me2.Detail, "boom")
			}

			// Errors.As must still reach the underlying *DaemonError
			// — callers who catch the broader type keep working.
			var de *DaemonError
			if !errors.As(me, &de) {
				t.Fatalf("errors.As(*MigrationError, *DaemonError) must succeed")
			}
			if !strings.HasPrefix(de.Error(), "daemon: migration: ") {
				t.Errorf("wrapped DaemonError prefix broken: %q", de.Error())
			}
		})
	}

	// Unknown kind demotes to MigrationErrKindUnknown (forward
	// compat — newer Rust vocabulary must not crash older Go).
	d := &DaemonError{Message: "migration: future-kind-never-seen"}
	me := parseMigrationError(d)
	if me == nil {
		t.Fatal("parseMigrationError must not return nil for a well-formed migration: message with unknown kind")
	}
	if me.Kind != MigrationErrKindUnknown {
		t.Errorf("unknown kind = %q, want MigrationErrKindUnknown (%q)", me.Kind, MigrationErrKindUnknown)
	}

	// Non-migration bodies must not lift — returning nil lets
	// `migrationErr` fall back to the raw *DaemonError.
	d2 := &DaemonError{Message: "factory not found"}
	if parseMigrationError(d2) != nil {
		t.Errorf("parseMigrationError must return nil for a non-migration-prefixed message")
	}
	if parseMigrationError(nil) != nil {
		t.Errorf("parseMigrationError(nil) must return nil")
	}
}

