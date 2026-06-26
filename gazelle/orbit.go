// Package orbit contains the Gazelle language plugin (gazelle_orbit) that
// generates vhdl_library and verilog_library targets from HDL sources by
// invoking the bundled orbit binary as a parser/analyzer backend.
package orbit

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"sync"

	"github.com/bazelbuild/rules_go/go/runfiles"
)

// lockedDirs tracks which directories have already been `orbit lock`-ed
// during this Gazelle run, so we don't redundantly lock a shared dependency
// multiple times. The same plugin instance handles every directory the
// traversal visits, so this stays in-process for the whole `bazel run`.
var (
	lockedDirsMu sync.Mutex
	lockedDirs   = map[string]bool{}
)

// pathDepRegex matches `path = "<path>"` entries inside an Orbit.toml.
// Used by ensureLocked to discover sibling project dependencies that must
// be locked first.
var pathDepRegex = regexp.MustCompile(`(?m)^\s*[A-Za-z0-9_-]+\s*=\s*\{[^}]*\bpath\s*=\s*"([^"]+)"`)

// ensureLocked guarantees `dir` has an up-to-date Orbit.lock, recursively
// locking any path-based dependencies first. No-op for directories without
// an Orbit.toml.
func ensureLocked(bin, dir string) error {
	abs, err := filepath.Abs(dir)
	if err != nil {
		return err
	}

	lockedDirsMu.Lock()
	if lockedDirs[abs] {
		lockedDirsMu.Unlock()
		return nil
	}
	// Mark eagerly to break dependency cycles (orbit doesn't allow them,
	// but a misconfigured workspace shouldn't cause infinite recursion).
	lockedDirs[abs] = true
	lockedDirsMu.Unlock()

	manifestPath := filepath.Join(abs, "Orbit.toml")
	manifest, err := os.ReadFile(manifestPath)
	if err != nil {
		return nil
	}

	// Lock every path dependency before this project. `orbit lock` reads
	// each path dep's lockfile, so they must exist by the time we run it
	// here.
	for _, match := range pathDepRegex.FindAllStringSubmatch(string(manifest), -1) {
		depPath := match[1]
		if !filepath.IsAbs(depPath) {
			depPath = filepath.Join(abs, depPath)
		}
		if err := ensureLocked(bin, depPath); err != nil {
			return err
		}
	}

	lockCmd := exec.Command(bin, "-C", abs, "lock")
	if out, err := lockCmd.CombinedOutput(); err != nil {
		return fmt.Errorf("orbit lock failed in %s: %w\noutput: %s", abs, err, out)
	}
	return nil
}

// resolveOrbitBin picks the orbit executable to invoke. Priority:
//  1. explicit --orbit_bin flag (treated as a direct path, then as an
//     rlocationpath, in that order).
//  2. the rlocationpath baked in by the build system (orbitBinRlocation).
//  3. "orbit" on $PATH.
func resolveOrbitBin(orbitBinFlag string) string {
	if orbitBinFlag != "" {
		if _, err := os.Stat(orbitBinFlag); err == nil {
			return orbitBinFlag
		}
		if resolved, err := runfiles.Rlocation(orbitBinFlag); err == nil {
			return resolved
		}
	}
	if orbitBinRlocation != "" {
		if resolved, err := runfiles.Rlocation(orbitBinRlocation); err == nil {
			return resolved
		}
	}
	return "orbit"
}

// orbitBinRlocation is the runfiles-relative path to the orbit binary that
// gets shipped with this plugin. It is overridden at link time via the
// go_library's x_defs (see //gazelle:BUILD.bazel). At runtime we resolve it
// through the bazel runfiles library to get a real filesystem path.
//
// When the value is empty (e.g. when the plugin is built outside Bazel) we
// fall back to the orbit_bin flag, then to "orbit" on $PATH.
var orbitBinRlocation = ""

// RefInfo is a single HDL-level reference (use clause, import, instantiation).
// Library is nil for unqualified references (e.g. Verilog module instantiations).
type RefInfo struct {
	Library *string `json:"library"`
	Name    string  `json:"name"`
}

// UnitInfo is one design unit (entity, module, package, etc.) as reported by
// `orbit analyze --json`.
type UnitInfo struct {
	Name     string    `json:"name"`
	UnitType string    `json:"unit_type"`
	Language string    `json:"language"`
	Library  string    `json:"library"`
	Sources  []string  `json:"sources"`
	Refs     []RefInfo `json:"refs"`
}

// AnalyzeOutput matches the JSON schema emitted by `orbit analyze --json`.
type AnalyzeOutput struct {
	Units []UnitInfo `json:"units"`
}

// runOrbitAnalyze invokes the orbit binary in dir and parses its JSON output.
// orbitBin is the path to the orbit executable. When orbitBin is empty, "orbit"
// is looked up on $PATH.
func runOrbitAnalyze(orbitBin, dir string) (*AnalyzeOutput, error) {
	bin := resolveOrbitBin(orbitBin)

	// Make `bazel run //:gazelle` the only command a user needs to invoke:
	// ensure Orbit.lock is up-to-date before analyzing. Locking is
	// recursive — orbit refuses to lock a project until each of its path
	// dependencies has a lockfile of its own — so we walk path deps
	// depth-first and lock them in topological order.
	if err := ensureLocked(bin, dir); err != nil {
		return nil, err
	}

	cmd := exec.Command(bin, "-C", dir, "analyze", "--json")
	out, err := cmd.Output()
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return nil, fmt.Errorf("orbit analyze failed in %s: %w\nstderr: %s", dir, err, exitErr.Stderr)
		}
		return nil, fmt.Errorf("orbit analyze failed in %s: %w", dir, err)
	}
	var parsed AnalyzeOutput
	if err := json.Unmarshal(out, &parsed); err != nil {
		return nil, fmt.Errorf("failed to parse orbit JSON output: %w", err)
	}
	return &parsed, nil
}
