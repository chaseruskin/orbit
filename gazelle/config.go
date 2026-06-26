package orbit

import (
	"flag"
	"path/filepath"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

// orbitConfig is the per-directory configuration state for the plugin.
// Created fresh at the root and inherited (then mutated) for each subdirectory
// via Configure(). The state lives in config.Config.Exts under languageName.
type orbitConfig struct {
	// orbitBin is the path to the orbit executable. May be empty, in which
	// case "orbit" is looked up on $PATH.
	orbitBin string

	// libraryName overrides the HDL library name reported by orbit for any
	// units found in this directory. Empty means "use what orbit reports".
	libraryName string

	// ignoreUnits is the set of design-unit names to skip when generating
	// rules in this directory.
	ignoreUnits map[string]bool

	// disabled disables rule generation entirely for this directory subtree.
	disabled bool
}

func defaultConfig() *orbitConfig {
	return &orbitConfig{
		ignoreUnits: map[string]bool{},
	}
}

func (c *orbitConfig) clone() *orbitConfig {
	cp := &orbitConfig{
		orbitBin:    c.orbitBin,
		libraryName: c.libraryName,
		disabled:    c.disabled,
		ignoreUnits: make(map[string]bool, len(c.ignoreUnits)),
	}
	for k := range c.ignoreUnits {
		cp.ignoreUnits[k] = true
	}
	return cp
}

// getConfig returns the orbit config for c, creating a default if absent.
func getConfig(c *config.Config) *orbitConfig {
	if v, ok := c.Exts[languageName]; ok {
		return v.(*orbitConfig)
	}
	cfg := defaultConfig()
	c.Exts[languageName] = cfg
	return cfg
}

// RegisterFlags implements config.Configurer.
func (*orbitLang) RegisterFlags(fs *flag.FlagSet, cmd string, c *config.Config) {
	cfg := getConfig(c)
	fs.StringVar(&cfg.orbitBin, "orbit_bin", "", "Path to the orbit executable. Defaults to looking up `orbit` on $PATH.")
}

// CheckFlags implements config.Configurer.
func (*orbitLang) CheckFlags(fs *flag.FlagSet, c *config.Config) error {
	if cfg, ok := c.Exts[languageName]; ok {
		oc := cfg.(*orbitConfig)
		if oc.orbitBin != "" {
			abs, err := filepath.Abs(oc.orbitBin)
			if err == nil {
				oc.orbitBin = abs
			}
		}
	}
	return nil
}

// KnownDirectives implements config.Configurer.
//
// Note: Gazelle's built-in `# gazelle:resolve orbit <import> <label>`
// directive is the canonical way to map a unit reference to a Bazel label
// (vendor IP, external stdlibs, codegen wrappers). The resolver consults it
// in resolve.go via resolve.FindRuleWithOverride, so we don't declare it
// here.
func (*orbitLang) KnownDirectives() []string {
	return []string{
		"orbit_disable",
		"orbit_library",
		"orbit_ignore",
	}
}

// Configure implements config.Configurer. It is called for each directory
// during traversal; we inherit (clone) the parent's config then apply any
// directives found in this directory's BUILD file.
func (*orbitLang) Configure(c *config.Config, rel string, f *rule.File) {
	parent := getConfig(c)
	cfg := parent.clone()
	c.Exts[languageName] = cfg

	if f == nil {
		return
	}
	for _, d := range f.Directives {
		switch d.Key {
		case "orbit_disable":
			cfg.disabled = strings.EqualFold(strings.TrimSpace(d.Value), "true")
		case "orbit_library":
			cfg.libraryName = strings.TrimSpace(d.Value)
		case "orbit_ignore":
			for _, name := range strings.Fields(d.Value) {
				cfg.ignoreUnits[name] = true
			}
		}
	}
}
