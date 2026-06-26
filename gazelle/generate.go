package orbit

import (
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/language"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

// vhdlExts and verilogExts list the source extensions we accept per language.
var (
	vhdlExts          = map[string]bool{".vhd": true, ".vhdl": true}
	verilogExts       = map[string]bool{".v": true, ".vh": true}
	systemVerilogExts = map[string]bool{".sv": true, ".svh": true}
)

// ruleBucket groups all design units that share the same source file set
// (e.g. a VHDL entity + its architecture) into a single Bazel rule.
//
// Each bucket's `kind` is decided up front from the srcs' file extensions:
//   - all `.vhd` / `.vhdl` → `vhdl_library`
//   - all `.v` / `.sv` → `verilog_library`
//   - mixed extensions in one bucket → `hdl_library`
//
// A bucket initially generated as a single-language rule may still be
// upgraded to `hdl_library` at Resolve time if any resolved dep turns out
// to belong to the other language (see Resolve() in resolve.go).
type ruleBucket struct {
	ruleName string
	kind     string
	library  string
	srcs     []string
	units    []UnitInfo
}

// GenerateRules implements language.Language.
//
// Strategy: for any directory we decide to "own" (per shouldGenerateAtThisDir),
// invoke `orbit analyze --json` and translate the reported design units into
// vhdl_library / verilog_library rules — one per primary design unit, with
// srcs limited to files whose nearest ancestor BUILD is this BUILD (i.e.,
// files directly in this dir, or in subdirectories that don't have their
// own BUILD between them and us).
//
// Cross-package references (units defined in other directories) are left as
// raw import specs and resolved later in Resolve().
//
// Placement rule (the "nearest existing ancestor BUILD" rule): gazelle never
// invents a new sub-BUILD inside a subtree where an ancestor BUILD already
// owns the source files. This keeps `glob(["<subdir>/**"])`-style targets in
// the ancestor BUILD intact (no fragmenting of subtrees), which matters for
// vendor IP repos that ship as `vivado_packaged_ip(srcs = glob([...]))` and
// need a recursive glob to stay un-truncated.
func (*orbitLang) GenerateRules(args language.GenerateArgs) language.GenerateResult {
	cfg := getConfig(args.Config)
	if cfg.disabled {
		return language.GenerateResult{}
	}

	// One BUILD-presence cache shared across the three walkers below, so
	// every `hasBuildFile` lookup happens at most once per unique dir per
	// GenerateRules invocation. Re-created per call because Gazelle's
	// language interface is stateless and the same ancestor chain is
	// walked many times for a project with lots of HDL files in
	// `subdir/`-style layouts.
	cache := buildFileCache{}

	// Placement gate. If this dir has no BUILD of its own AND some ancestor
	// already does, defer to the ancestor — its GenerateRules call collects
	// our HDL via the subtree walk below. Skipping here means the framework
	// won't write a new BUILD into this subdir.
	if args.File == nil && anyAncestorHasBuild(args.Dir, args.Config.RepoRoot, cache) {
		return language.GenerateResult{}
	}

	// HDL discovery has to be subtree-aware: this dir may own HDL files in
	// descendant directories that don't have their own BUILDs (the
	// vendor-IP case — `ip/BUILD.bazel` owns HDL nested several levels deep
	// under `managed_ethernet_switch/hdl/mes_hdl/...`). `args.RegularFiles`
	// only contains files in this directory, so we walk the subtree
	// ourselves, stopping at any descendant BUILD (those subtrees are
	// owned by their own GenerateRules calls).
	if !hasOwnedHDLInSubtree(args.Dir, cache) {
		return language.GenerateResult{}
	}

	parsed, err := runOrbitAnalyze(cfg.orbitBin, args.Dir)
	if err != nil {
		log.Printf("gazelle_orbit: skipping %s: %v", args.Rel, err)
		return language.GenerateResult{}
	}

	// Group units that share the same source file set into one rule so we
	// don't generate two targets with identical srcs (e.g. a VHDL entity
	// and its architecture). Key on a canonical sorted srcs list.
	buckets := map[string]*ruleBucket{}
	bucketOrder := []string{}

	for _, u := range parsed.Units {
		if cfg.ignoreUnits[u.Name] {
			continue
		}
		localSrcs := filesUnderBuildOwnership(u.Sources, args.Dir, cache)
		if len(localSrcs) == 0 {
			continue
		}
		if !hasSupportedLanguage(u.Language) {
			continue
		}
		library := u.Library
		if cfg.libraryName != "" {
			library = cfg.libraryName
		}
		// Bucket by (library, srcs) only — kind is derived from srcs
		// extensions after bucketing so a single bucket whose srcs span
		// both languages becomes one `hdl_library` rather than splitting
		// into two redundant targets.
		key := library + "|" + strings.Join(localSrcs, ",")
		b, ok := buckets[key]
		if !ok {
			b = &ruleBucket{
				ruleName: ruleNameFor(u),
				library:  library,
				srcs:     localSrcs,
			}
			buckets[key] = b
			bucketOrder = append(bucketOrder, key)
		}
		b.units = append(b.units, u)
	}

	result := language.GenerateResult{}
	for _, k := range bucketOrder {
		b := buckets[k]
		b.kind = kindFromSrcs(b.srcs)
		if b.kind == "" {
			// Bucket had no recognized HDL extensions — skip.
			continue
		}
		// Honor a user's prior manual upgrade: if the BUILD file already
		// declares this rule as `hdl_library`, keep generating it as
		// `hdl_library` even if srcs are single-language. Otherwise the
		// gen rule's kind wouldn't match the existing kind and merger
		// would reject it the same way a Resolve-time upgrade does.
		if existing := findExistingRuleByName(args.File, b.ruleName); existing != nil && existing.Kind() == kindHdlLibrary {
			b.kind = kindHdlLibrary
		}
		r := rule.NewRule(b.kind, b.ruleName)
		r.SetAttr("srcs", b.srcs)
		// HDL libraries are almost always consumed across packages
		// (cpu/dsp depending on primitives, vutils, gates, etc.) so we
		// default-emit public visibility. If a user wants narrower
		// scoping they can edit the BUILD file and Gazelle will leave
		// their override in place.
		r.SetAttr("visibility", []string{"//visibility:public"})
		// Every plugin-generated rule carries the `gazelle_orbit` tag so
		// users can tell at a glance which targets are auto-managed (and
		// query/filter on them, e.g. `bazel query 'attr(tags,
		// "\bgazelle_orbit\b", //...)'`). User-added tags survive
		// because we union-merge with what's already on the rule.
		r.SetAttr("tags", mergeOrbitTag(existingTagsForRule(args.File, b.ruleName)))
		// Record whether the BUILD file already has a rule with this
		// name. If it does, Resolve must NOT upgrade the kind
		// (`verilog_library` → `hdl_library` on cross-language dep)
		// because gazelle's `merger.Match` rejects same-name-different-
		// kind pairs and would silently drop the deps update entirely.
		// Resolve instead logs a one-line warning so the user can hand-
		// migrate the rule kind, after which subsequent gazelle runs
		// merge cleanly.
		if findExistingRuleByName(args.File, b.ruleName) != nil {
			r.SetPrivateAttr(noKindUpgradeAttr, true)
		}
		// VHDL and mixed-language libraries get an explicit `library = ...`
		// attribute. `verilog_library` has no public `library` attr
		// (rules_verilog: Verilog's source-level namespace is flat); we
		// stash the project's HDL library name in a private attr so
		// Imports() can still index `<library>.<unit>`-qualified refs and
		// so a later Resolve-time upgrade to `hdl_library` can promote it
		// to a public attr.
		if b.kind == kindVerilogLibrary {
			r.SetPrivateAttr(libraryNameAttr, b.library)
		} else {
			r.SetAttr("library", b.library)
		}
		// Stash the list of design-unit names this rule provides so
		// Imports() can index each one independently. Gazelle preserves
		// private attrs but never writes them to the BUILD file.
		r.SetPrivateAttr(unitNamesAttr, unitNamesIn(b))
		result.Gen = append(result.Gen, r)
		// Build the import-spec slice that Resolve() will receive.
		result.Imports = append(result.Imports, importsForBucket(b))
	}

	return result
}

// kindFromSrcs decides the rule kind for a bucket from the file extensions
// in its srcs. Mixed-language buckets become `hdl_library` so the unified
// rule's provider partitioning handles both languages from one target.
func kindFromSrcs(srcs []string) string {
	hasVhdl := false
	hasVerilog := false
	for _, s := range srcs {
		ext := strings.ToLower(filepath.Ext(s))
		if vhdlExts[ext] {
			hasVhdl = true
		} else if verilogExts[ext] || systemVerilogExts[ext] {
			hasVerilog = true
		}
	}
	switch {
	case hasVhdl && hasVerilog:
		return kindHdlLibrary
	case hasVhdl:
		return kindVhdlLibrary
	case hasVerilog:
		return kindVerilogLibrary
	default:
		return ""
	}
}

// hasSupportedLanguage reports whether orbit's reported language string is
// one we can map to a Bazel rule. Replaces the old `kindForLanguage`
// helper — kind is now derived from srcs extensions, not the per-unit
// language string.
func hasSupportedLanguage(lang string) bool {
	switch strings.ToLower(lang) {
	case "vhdl", "verilog", "systemverilog":
		return true
	default:
		return false
	}
}

// libraryNameAttr is the private attribute key used to stash the HDL library
// name on Verilog rules (where there is no public `library` attribute to
// read it from).
const libraryNameAttr = "_orbit_library"

// orbitTag is the bare tag value attached to every plugin-generated rule.
// Distinct from the prefixed tags (`orbit_library=…`, `orbit_unit=…`) that
// are read off hand-authored rules — this one is purely a "produced by
// gazelle_orbit" marker.
const orbitTag = "gazelle_orbit"

// existingTagsForRule reads any prior tags off a rule that already exists
// in the BUILD file under the given name. Returns nil when the rule is new.
func existingTagsForRule(f *rule.File, name string) []string {
	if r := findExistingRuleByName(f, name); r != nil {
		return r.AttrStrings("tags")
	}
	return nil
}

// mergeOrbitTag returns existing ∪ {orbitTag}, deduplicated and sorted.
// Used to ensure the marker is always present on plugin-generated rules
// without clobbering user-added tags on re-runs.
func mergeOrbitTag(existing []string) []string {
	seen := map[string]bool{orbitTag: true}
	out := []string{orbitTag}
	for _, t := range existing {
		if seen[t] {
			continue
		}
		seen[t] = true
		out = append(out, t)
	}
	sort.Strings(out)
	return out
}

// unitNamesIn returns the sanitized names of every design unit in b.
func unitNamesIn(b *ruleBucket) []string {
	out := make([]string, 0, len(b.units))
	seen := map[string]bool{}
	for _, u := range b.units {
		n := sanitizeName(u.Name)
		if seen[n] {
			continue
		}
		seen[n] = true
		out = append(out, n)
	}
	return out
}

// unitNamesAttr is the private attribute key used to stash the bucket's
// design-unit names on each generated rule.
const unitNamesAttr = "_orbit_unit_names"

// noKindUpgradeAttr marks a generated rule whose name already exists in
// the BUILD file. Resolve checks this flag before attempting a kind
// upgrade — when the flag is set, Resolve emits a warning and leaves the
// kind alone so the merger doesn't reject the rule and drop its deps.
const noKindUpgradeAttr = "_orbit_no_kind_upgrade"

// findExistingRuleByName returns the existing rule of the given name in
// the BUILD file, or nil if none exists.
func findExistingRuleByName(f *rule.File, name string) *rule.Rule {
	if f == nil {
		return nil
	}
	for _, r := range f.Rules {
		if r.Name() == name {
			return r
		}
	}
	return nil
}

// importsForBucket flattens all refs from all units in a bucket into a single
// dedup'd slice, exclusive of self-imports (refs pointing to a unit defined
// in the same bucket).
func importsForBucket(b *ruleBucket) []ruleImport {
	selfUnits := map[string]bool{}
	for _, u := range b.units {
		selfUnits[sanitizeName(u.Name)] = true
	}
	seen := map[string]bool{}
	out := []ruleImport{}
	for _, u := range b.units {
		for _, r := range u.Refs {
			lib := ""
			if r.Library != nil {
				lib = *r.Library
			}
			name := sanitizeName(r.Name)
			key := strings.ToLower(lib) + "." + name
			if seen[key] {
				continue
			}
			// Skip self-references and refs to units defined in the
			// same bucket.
			if selfUnits[name] {
				continue
			}
			seen[key] = true
			out = append(out, ruleImport{Library: lib, Name: name, OriginLib: b.library})
		}
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Library != out[j].Library {
			return out[i].Library < out[j].Library
		}
		return out[i].Name < out[j].Name
	})
	return out
}

// pickHDLFiles returns the subset of files in regularFiles that have an HDL
// extension.
func pickHDLFiles(regularFiles []string) []string {
	var out []string
	for _, f := range regularFiles {
		ext := strings.ToLower(filepath.Ext(f))
		if vhdlExts[ext] || verilogExts[ext] || systemVerilogExts[ext] {
			out = append(out, f)
		}
	}
	return out
}

// filesUnderBuildOwnership returns the subset of absolute paths in srcs
// that belong to the BUILD at buildDir per the nearest-existing-ancestor-
// BUILD rule, with paths rewritten relative to buildDir so they're valid
// Bazel srcs entries.
//
// A file belongs to buildDir iff every directory between the file's parent
// and buildDir (inclusive of intermediate dirs, exclusive of buildDir) has
// NO `BUILD.bazel` / `BUILD`. The first intervening BUILD encountered while
// walking up from the file owns it instead.
//
// For files that are direct children of buildDir, the rewritten path is
// just the basename (no slashes). For files in subdirectories, it's the
// sub-path relative to buildDir (e.g. `subdir/foo.vhd`).
func filesUnderBuildOwnership(srcs []string, buildDir string, cache buildFileCache) []string {
	absDir, err := filepath.Abs(buildDir)
	if err != nil {
		return nil
	}
	var out []string
	for _, s := range srcs {
		absS, err := filepath.Abs(s)
		if err != nil {
			continue
		}
		if !fileBelongsToBuild(absS, absDir, cache) {
			continue
		}
		rel, err := filepath.Rel(absDir, absS)
		if err != nil {
			continue
		}
		out = append(out, filepath.ToSlash(rel))
	}
	sort.Strings(out)
	return out
}

// fileBelongsToBuild reports whether absFile (absolute path) should be
// owned by the BUILD at buildDirAbs (also absolute) per the nearest-
// existing-ancestor-BUILD rule. Returns false if the file is outside
// buildDirAbs's subtree OR if any directory strictly between the file's
// parent and buildDirAbs (exclusive of buildDirAbs) holds its own
// BUILD.bazel / BUILD.
func fileBelongsToBuild(absFile, buildDirAbs string, cache buildFileCache) bool {
	dir := filepath.Dir(absFile)
	// Must be a descendant (or same dir).
	rel, err := filepath.Rel(buildDirAbs, dir)
	if err != nil || rel == ".." || strings.HasPrefix(rel, ".."+string(filepath.Separator)) {
		return false
	}
	for dir != buildDirAbs {
		if cache.has(dir) {
			return false
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			// Reached filesystem root without hitting buildDirAbs — file
			// wasn't actually under buildDirAbs (defensive; the
			// filepath.Rel check above should have caught it).
			return false
		}
		dir = parent
	}
	return true
}

// anyAncestorHasBuild reports whether any directory strictly between
// startDir (exclusive) and repoRoot (inclusive) holds a BUILD.bazel /
// BUILD. Used to decide whether GenerateRules should defer this dir's
// rules to an ancestor BUILD.
func anyAncestorHasBuild(startDir, repoRoot string, cache buildFileCache) bool {
	abs, err := filepath.Abs(startDir)
	if err != nil {
		return false
	}
	root, err := filepath.Abs(repoRoot)
	if err != nil {
		return false
	}
	dir := filepath.Dir(abs)
	for {
		if cache.has(dir) {
			return true
		}
		if dir == root {
			return false
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return false
		}
		dir = parent
	}
}

// buildFileCache memoises hasBuildFile lookups within a single
// GenerateRules call. The three walkers (anyAncestorHasBuild upward,
// fileBelongsToBuild upward-bounded, hasOwnedHDLInSubtree downward) all
// stat the same ancestor chains repeatedly when a project has many
// HDL files under one BUILD — the cache turns N walks × M ancestors
// of redundant `os.Stat` calls into one stat per unique dir per call.
type buildFileCache map[string]bool

func (c buildFileCache) has(dirAbs string) bool {
	if v, ok := c[dirAbs]; ok {
		return v
	}
	v := hasBuildFile(dirAbs)
	c[dirAbs] = v
	return v
}

// hasBuildFile reports whether dir contains a `BUILD.bazel` or `BUILD`
// file. Prefer the per-call `buildFileCache.has` over the raw form
// when called from a walk that will revisit the same ancestors.
func hasBuildFile(dirAbs string) bool {
	for _, name := range []string{"BUILD.bazel", "BUILD"} {
		if _, err := os.Stat(filepath.Join(dirAbs, name)); err == nil {
			return true
		}
	}
	return false
}

// hasOwnedHDLInSubtree reports whether any HDL source file under buildDir
// would be owned by buildDir per the nearest-existing-ancestor-BUILD rule
// — i.e., any HDL file in buildDir itself OR in a descendant subdirectory
// whose chain back up to buildDir contains no intervening BUILD.bazel /
// BUILD. Used as an early-out: if the answer is no, GenerateRules can
// return without running orbit analyze.
//
// The walk SKIPS any subdirectory that has its own BUILD.bazel / BUILD —
// those are their own packages and their HDL is owned by their own
// GenerateRules call.
func hasOwnedHDLInSubtree(buildDir string, cache buildFileCache) bool {
	found := false
	_ = filepath.WalkDir(buildDir, func(path string, d os.DirEntry, err error) error {
		if err != nil || found {
			return filepath.SkipDir
		}
		if d.IsDir() {
			// Don't descend into a directory that has its own BUILD —
			// it's a separate package owned by its own GenerateRules.
			if path != buildDir && cache.has(path) {
				return filepath.SkipDir
			}
			return nil
		}
		ext := strings.ToLower(filepath.Ext(d.Name()))
		if vhdlExts[ext] || verilogExts[ext] || systemVerilogExts[ext] {
			found = true
			return filepath.SkipAll
		}
		return nil
	})
	return found
}

// ruleNameFor derives the Bazel target name for a unit. We lowercase and
// strip the VHDL extended-identifier wrapping (`\name `) so the result is a
// legal Bazel label.
func ruleNameFor(u UnitInfo) string {
	return sanitizeName(u.Name)
}

func sanitizeName(s string) string {
	s = strings.TrimSpace(s)
	s = strings.TrimPrefix(s, `\`)
	s = strings.TrimSuffix(s, `\`)
	s = strings.TrimSpace(s)
	return strings.ToLower(s)
}
