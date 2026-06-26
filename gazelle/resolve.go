package orbit

import (
	"log"
	"sort"
	"strings"
	"sync"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/repo"
	"github.com/bazelbuild/bazel-gazelle/resolve"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

// HDL language tags used to drive Resolve-time cross-language detection.
// A rule's language is the union of providers it emits: a `vhdl_library`
// provides only `languageVhdl`; a `verilog_library` provides only
// `languageVerilog`; an `hdl_library` may provide both.
const (
	languageVhdl    = "vhdl"
	languageVerilog = "verilog"
)

// labelLanguages tracks which HDL language(s) each generated rule provides.
// Imports() writes to it for every plugin-managed rule it sees in the
// workspace; Resolve() reads from it to classify the language of each
// resolved dep label.
//
// Keys are normalized to "pkg:name" (no repo prefix) because Imports()
// doesn't see the apparent repo name attached to its rules while Resolve()
// gets labels with the consumer's repo baked in. Cross-workspace cross-
// language refs aren't a real concern (different workspaces have separate
// HDL dep graphs); within a single workspace, pkg:name uniquely identifies
// each rule.
//
// Gazelle's processing model serializes Imports() across all rules before
// any Resolve() call runs, so by the time we read this map every relevant
// entry is present. We use sync.Map so the Imports phase can run
// concurrently across packages without an explicit mutex.
var labelLanguages sync.Map // map[string]map[string]bool — "pkg:name" → set{"vhdl","verilog"}

// labelKey produces the normalized key used by labelLanguages — strips the
// repo prefix so Imports-time and Resolve-time labels match regardless of
// whether the apparent repo name was attached.
func labelKey(lbl label.Label) string {
	return lbl.Pkg + ":" + lbl.Name
}

// recordLabelLanguages stores the set of languages a rule's label provides.
// Called from Imports(); overwrites previous entries (idempotent on
// re-resolve).
func recordLabelLanguages(lbl label.Label, langs map[string]bool) {
	if len(langs) == 0 {
		return
	}
	labelLanguages.Store(labelKey(lbl), langs)
}

// lookupLabelLanguages returns the language set associated with lbl, or nil
// if the label wasn't indexed (typically because it's an external label or
// a hand-authored rule not in our codegen list).
func lookupLabelLanguages(lbl label.Label) map[string]bool {
	v, ok := labelLanguages.Load(labelKey(lbl))
	if !ok {
		return nil
	}
	return v.(map[string]bool)
}

// languagesForKind returns the set of HDL languages a rule of the given
// kind provides. For codegen kinds whose language is ambiguous from the
// kind name (e.g. `verilog_system_rdl_library` is Verilog/SV; future
// codegens may be VHDL), we conservatively return both languages — the
// kind-upgrade heuristic in Resolve treats "unknown" as "matches the
// consumer" so we never spuriously upgrade based on a codegen dep.
func languagesForKind(kind string) map[string]bool {
	switch kind {
	case kindVhdlLibrary:
		return map[string]bool{languageVhdl: true}
	case kindVerilogLibrary, "verilog_system_rdl_library":
		return map[string]bool{languageVerilog: true}
	case kindHdlLibrary:
		return map[string]bool{languageVhdl: true, languageVerilog: true}
	default:
		return nil
	}
}

// labelFromRule constructs the canonical label for a rule given the file it
// lives in. Mirrors how Bazel forms labels: `//<f.Pkg>:<r.Name()>` in the
// main repo, no apparent-name prefix.
func labelFromRule(r *rule.Rule, f *rule.File) label.Label {
	pkg := ""
	if f != nil {
		pkg = f.Pkg
	}
	return label.New("", pkg, r.Name())
}

// ruleImport is the per-import data we attached to each generated rule in
// GenerateRules; Resolve() receives this as the `imports interface{}`.
//
// Library: the qualifying library name from the HDL source (e.g. "ieee" or
// the work library), or empty for unqualified Verilog refs.
// Name: the design-unit name being referenced.
// OriginLib: the HDL library of the rule that owns this import — used to
// resolve the VHDL `work` library back to a concrete library name.
type ruleImport struct {
	Library   string
	Name      string
	OriginLib string
}

// Imports implements resolve.Resolver. It tells Gazelle which import keys
// this rule provides (so other rules' refs can be matched to it).
//
// We index each design unit twice: once unqualified ("<unit_name>") for
// Verilog-style instantiations, and once library-qualified
// ("<library>.<unit_name>") for VHDL use-clauses.
//
// Three sources contribute unit names:
//  1. The private attr stashed by GenerateRules — authoritative for rules
//     the plugin emitted.
//  2. `orbit_unit=<name>` entries in the rule's `tags` attribute — the
//     mechanism a hand-author uses to declare design units provided by
//     a wrapper rule around a code generator (e.g. SystemRDL → vhdl_library).
//  3. The rule's `name` — used either as the sole unit name when no other
//     source is present, or always merged in alongside the others so the
//     conventional case (one rule == one unit) needs no annotation.
func (*orbitLang) Imports(c *config.Config, r *rule.Rule, f *rule.File) []resolve.ImportSpec {
	// For vhdl_library the library name lives on the public `library` attr.
	// For verilog_library (no such attr in rules_verilog), the plugin
	// stashes it in a private attr at generation time. Hand-authored
	// verilog_library rules that want to participate in library-qualified
	// lookups can declare the name via `tags = ["orbit_library=<name>"]`.
	library := strings.ToLower(r.AttrString("library"))
	if library == "" {
		if priv, ok := r.PrivateAttr(libraryNameAttr).(string); ok {
			library = strings.ToLower(priv)
		}
	}
	if library == "" {
		for _, tag := range r.AttrStrings("tags") {
			if rest, ok := strings.CutPrefix(tag, tagOrbitLibraryPrefix); ok {
				library = strings.ToLower(rest)
				break
			}
		}
	}

	seen := map[string]bool{}
	var names []string
	add := func(n string) {
		n = strings.ToLower(n)
		if n == "" || seen[n] {
			return
		}
		seen[n] = true
		names = append(names, n)
	}

	if pluginNames, ok := r.PrivateAttr(unitNamesAttr).([]string); ok {
		for _, n := range pluginNames {
			add(n)
		}
	}
	for _, tag := range r.AttrStrings("tags") {
		if rest, ok := strings.CutPrefix(tag, tagOrbitUnitPrefix); ok {
			add(rest)
		}
	}
	add(r.Name())

	specs := make([]resolve.ImportSpec, 0, len(names)*2)
	for _, n := range names {
		specs = append(specs, resolve.ImportSpec{Lang: languageName, Imp: n})
		if library != "" {
			specs = append(specs, resolve.ImportSpec{
				Lang: languageName,
				Imp:  library + "." + n,
			})
		}
	}

	// Record this rule's language set for Resolve-time cross-language
	// detection. The plugin's `hdl_library` provides both; the
	// single-language kinds provide just one; codegen kinds resolve via
	// `languagesForKind` (currently Verilog/SV).
	recordLabelLanguages(labelFromRule(r, f), languagesForKind(r.Kind()))

	return specs
}

// tagOrbitUnitPrefix is the prefix used inside a rule's `tags = [...]` to
// declare additional design-unit names the rule provides. Used by
// hand-authored wrappers around code generators where the rule name only
// names one of several emitted units.
//
// Example:
//
//	vhdl_library(
//	    name = "uart_regs",
//	    srcs = [":uart_regs_src"],
//	    library = "uart_regs",
//	    tags = [
//	        "orbit_unit=uart_regs_pkg",
//	        "orbit_unit=uart_regs_decoder",
//	    ],
//	)
//
// Downstream HDL that says `use uart_regs.uart_regs_pkg;` (or any of the
// listed units) will resolve to this rule.
const tagOrbitUnitPrefix = "orbit_unit="

// tagOrbitLibraryPrefix declares the HDL library a hand-authored
// verilog_library belongs to. rules_verilog's verilog_library has no public
// `library` attribute (Verilog's source-level namespace is flat), so this
// tag is the way to participate in library-qualified resolution.
//
// Example:
//
//	verilog_library(
//	    name = "uart_regs",
//	    srcs = [":uart_regs_verilog"],
//	    tags = [
//	        "orbit_library=uart_regs",
//	        "orbit_unit=uart_regs_pkg",
//	    ],
//	)
const tagOrbitLibraryPrefix = "orbit_library="

// Resolve implements resolve.Resolver. For each import on the rule, we look
// up a matching Bazel label (either from a configured stdlib mapping or by
// querying the workspace index) and set the result on the rule's `deps`
// attribute.
//
// Resolve also performs the cross-language kind upgrade: if a rule was
// initially generated as `vhdl_library` or `verilog_library` (based on its
// own srcs language) but one of its resolved deps provides only the OTHER
// language, the rule is rewritten to `hdl_library` in place. The
// `hdl_library` rule's impl walks `[VhdlInfo]` and `[VerilogInfo]` on each
// dep independently, so cross-language refs participate in the real
// provider chain rather than the leaf-only `data = [...]` workaround.
func (*orbitLang) Resolve(c *config.Config, ix *resolve.RuleIndex, rc *repo.RemoteCache, r *rule.Rule, imports interface{}, from label.Label) {
	imps, ok := imports.([]ruleImport)
	if !ok || len(imps) == 0 {
		return
	}

	// If the BUILD file already has a single-language rule with this
	// name, Resolve can't upgrade its kind (merger would reject the
	// mismatch). In that case we DEFER cross-language deps too: writing
	// a vhdl_library label into `verilog_library.deps` would fail
	// `bazel build` with a VerilogInfo provider error, which is louder
	// than a missing dep but blocks unrelated work. Better to log a
	// one-line warning + skip the dep; the user upgrades the rule kind
	// manually, re-runs gazelle, and the cross-language dep lands
	// cleanly. Same-language deps still flow through normally.
	upgradeLocked, _ := r.PrivateAttr(noKindUpgradeAttr).(bool)
	ownLang := ownLanguageForKind(r.Kind())

	depSet := map[string]bool{}
	// Track the language profile of each resolved dep so we can decide
	// whether this rule needs to be upgraded to `hdl_library`. Keyed by
	// canonical absolute label string so we don't double-classify a dep
	// that's referenced via multiple imports.
	resolvedLangs := map[string]map[string]bool{}
	deferredCrossLang := []label.Label{}
	for _, imp := range imps {
		lbl, ok := resolveImport(c, ix, imp, from)
		if !ok {
			continue
		}
		if lbl == from || (lbl.Pkg == from.Pkg && lbl.Name == from.Name) {
			continue
		}
		absLbl := lbl.String()
		if _, seen := resolvedLangs[absLbl]; !seen {
			resolvedLangs[absLbl] = lookupLabelLanguages(lbl)
		}
		if upgradeLocked && ownLang != "" && hasCrossLanguageContribution(resolvedLangs[absLbl], ownLang) {
			deferredCrossLang = append(deferredCrossLang, lbl)
			continue
		}
		depSet[lbl.Rel(from.Repo, from.Pkg).String()] = true
	}
	for _, lbl := range deferredCrossLang {
		log.Printf(
			"gazelle_orbit: %s references %s across HDL languages but the BUILD "+
				"file already declares a `%s` rule with this name; cannot wire the "+
				"dep without upgrading the rule kind. Change it to `hdl_library` "+
				"(load from `@gazelle_orbit//hdl:defs.bzl`) and re-run gazelle to "+
				"pick up the cross-language dep.",
			from.String(), lbl.String(), r.Kind(),
		)
	}
	if len(depSet) == 0 {
		return
	}
	deps := make([]string, 0, len(depSet))
	for d := range depSet {
		deps = append(deps, d)
	}
	sort.Strings(deps)
	r.SetAttr("deps", deps)

	maybeUpgradeKind(r, resolvedLangs)
}

// ownLanguageForKind returns the single language a single-language plugin-
// generated rule provides, or empty for kinds that span both (hdl_library)
// or that the plugin doesn't manage.
func ownLanguageForKind(kind string) string {
	switch kind {
	case kindVhdlLibrary:
		return languageVhdl
	case kindVerilogLibrary:
		return languageVerilog
	default:
		return ""
	}
}

// hasCrossLanguageContribution reports whether the candidate language set
// includes a language OTHER than `ownLang` — which means an upgrade to
// `hdl_library` is required so the chain forwards that language to the
// rule's consumers.
//
// Returning true for deps providing BOTH languages (e.g. an
// `hdl_library`) is intentional: a `verilog_library` consuming an
// `hdl_library` only forwards the `VerilogInfo` half via its own
// `VerilogInfo.deps` depset; the VHDL chain dies there unless this rule
// is upgraded to `hdl_library` so it propagates both providers.
func hasCrossLanguageContribution(langs map[string]bool, ownLang string) bool {
	if len(langs) == 0 {
		return false
	}
	for lang := range langs {
		if lang != ownLang {
			return true
		}
	}
	return false
}

// maybeUpgradeKind rewrites the rule to `hdl_library` if any resolved dep
// provides only the language opposite to the rule's own. Same-language
// deps (whether VHDL→VHDL or Verilog→Verilog) don't trigger an upgrade.
// Deps with unknown language (external repos, hand-authored rules without
// our plugin's marker) are treated as same-language matches so we never
// upgrade based on incomplete information.
//
// When upgrading from `verilog_library` we promote the private
// `_orbit_library` attribute to a public `library` attr since
// `hdl_library` (unlike `verilog_library`) accepts `library` as a public
// attribute that names the VHDL library its .vhd srcs compile into.
//
// `noKindUpgradeAttr`-flagged rules are skipped: gazelle's `merger.Match`
// would reject a same-name-different-kind merge and silently drop the
// rule's deps. The cross-language deps for these rules are already
// deferred + warned in `Resolve` itself, so this is a quiet no-op here.
func maybeUpgradeKind(r *rule.Rule, resolvedLangs map[string]map[string]bool) {
	currentKind := r.Kind()
	if currentKind == kindHdlLibrary {
		// Already mixed-capable — nothing to upgrade.
		return
	}
	if locked, _ := r.PrivateAttr(noKindUpgradeAttr).(bool); locked {
		return
	}
	ownLang := ownLanguageForKind(currentKind)
	if ownLang == "" {
		// Unknown / non-plugin-generated kind — leave alone.
		return
	}

	for _, langs := range resolvedLangs {
		if !hasCrossLanguageContribution(langs, ownLang) {
			continue
		}
		r.SetKind(kindHdlLibrary)
		if currentKind == kindVerilogLibrary {
			if priv, ok := r.PrivateAttr(libraryNameAttr).(string); ok && priv != "" {
				r.SetAttr("library", priv)
			}
		}
		return
	}
}

// resolveImport finds the Bazel label that provides imp.
//
// Resolution order:
//  1. Gazelle's canonical `# gazelle:resolve orbit <import> <label>` override,
//     via resolve.FindRuleWithOverride. Used for vendor IP, external
//     stdlibs, and any "this import maps to this label" mapping.
//  2. Library-qualified workspace index lookup ("<library>.<unit>"). For
//     VHDL refs with library "work", substitute the origin rule's library
//     first.
//  3. Unqualified workspace index lookup ("<unit>") — typical Verilog.
func resolveImport(c *config.Config, ix *resolve.RuleIndex, imp ruleImport, from label.Label) (label.Label, bool) {
	library := imp.Library
	if strings.EqualFold(library, "work") {
		library = imp.OriginLib
	}

	qualifiedKey := strings.ToLower(library + "." + imp.Name)
	unqualifiedKey := strings.ToLower(imp.Name)

	// 1. Honor `# gazelle:resolve orbit <import> <label>` overrides. Try
	// the qualified form first, then unqualified — so users can override
	// at whichever granularity makes sense.
	for _, key := range []string{qualifiedKey, unqualifiedKey} {
		if key == "" {
			continue
		}
		if lbl, ok := resolve.FindRuleWithOverride(c, resolve.ImportSpec{
			Lang: languageName,
			Imp:  key,
		}, languageName); ok {
			return lbl, true
		}
	}

	// 2. Library-qualified workspace index lookup.
	if library != "" {
		results := ix.FindRulesByImportWithConfig(c, resolve.ImportSpec{
			Lang: languageName,
			Imp:  qualifiedKey,
		}, languageName)
		if r, ok := pickFirstNonSelf(results, from); ok {
			return r, true
		}
	}

	// 3. Unqualified workspace index lookup.
	results := ix.FindRulesByImportWithConfig(c, resolve.ImportSpec{
		Lang: languageName,
		Imp:  unqualifiedKey,
	}, languageName)
	if r, ok := pickFirstNonSelf(results, from); ok {
		return r, true
	}

	return label.NoLabel, false
}

// pickFirstNonSelf returns the first label that is not a self-import.
func pickFirstNonSelf(results []resolve.FindResult, from label.Label) (label.Label, bool) {
	for _, res := range results {
		if res.IsSelfImport(from) {
			continue
		}
		return res.Label, true
	}
	return label.NoLabel, false
}
