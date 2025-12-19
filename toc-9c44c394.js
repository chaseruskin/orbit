// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="index.html">Introduction</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="starting/starting.html"><strong aria-hidden="true">1.</strong> Getting Started</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="starting/installing.html"><strong aria-hidden="true">1.1.</strong> Installing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="starting/upgrading.html"><strong aria-hidden="true">1.2.</strong> Upgrading</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="tutorials/tutorials.html"><strong aria-hidden="true">2.</strong> Tutorials</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="tutorials/first_project.html"><strong aria-hidden="true">2.1.</strong> First Project: Gates</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="tutorials/dependencies.html"><strong aria-hidden="true">2.2.</strong> Dependencies: Half Adder</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="tutorials/gates_revisited.html"><strong aria-hidden="true">2.3.</strong> Gates: Revisited</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="tutorials/final_project.html"><strong aria-hidden="true">2.4.</strong> Final Project: Full Adder</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/user.html"><strong aria-hidden="true">3.</strong> User Guide</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/overview.html"><strong aria-hidden="true">3.1.</strong> How to Use Orbit</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/researching_projects.html"><strong aria-hidden="true">3.2.</strong> Researching Projects</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/developing_projects.html"><strong aria-hidden="true">3.3.</strong> Developing Projects</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/releasing_projects.html"><strong aria-hidden="true">3.4.</strong> Releasing Projects</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/creating_targets.html"><strong aria-hidden="true">3.5.</strong> Creating Targets</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/creating_protocols.html"><strong aria-hidden="true">3.6.</strong> Creating Protocols</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/creating_channels.html"><strong aria-hidden="true">3.7.</strong> Creating Channels</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="user/specifying_deps.html"><strong aria-hidden="true">3.8.</strong> Specifying Dependencies</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/topic.html"><strong aria-hidden="true">4.</strong> Topic Guide</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/why_orbit.html"><strong aria-hidden="true">4.1.</strong> Why Orbit Exists</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/overview.html"><strong aria-hidden="true">4.2.</strong> How Orbit Works</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/package_management.html"><strong aria-hidden="true">4.3.</strong> Package Management</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/extensible_builds.html"><strong aria-hidden="true">4.4.</strong> Extensible Builds</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/packages_and_cores.html"><strong aria-hidden="true">4.5.</strong> Packages and Cores</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/projects.html"><strong aria-hidden="true">4.6.</strong> Projects</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/catalog.html"><strong aria-hidden="true">4.7.</strong> Catalog</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/targets.html"><strong aria-hidden="true">4.8.</strong> Targets</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/protocols.html"><strong aria-hidden="true">4.9.</strong> Protocols</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/channels.html"><strong aria-hidden="true">4.10.</strong> Channels</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/orbitlock.html"><strong aria-hidden="true">4.11.</strong> Orbit.lock</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/core_visibility.html"><strong aria-hidden="true">4.12.</strong> IP Core Visibility</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/swapping.html"><strong aria-hidden="true">4.13.</strong> String Swapping</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="topic/dst.html"><strong aria-hidden="true">4.14.</strong> Dynamic Symbol Transformation</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/reference.html"><strong aria-hidden="true">5.</strong> Reference</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/manifest.html"><strong aria-hidden="true">5.1.</strong> Manifest</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/project_id_specification.html"><strong aria-hidden="true">5.2.</strong> Project ID Specification</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/versions.html"><strong aria-hidden="true">5.3.</strong> Versions</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/filesets.html"><strong aria-hidden="true">5.4.</strong> Filesets</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/blueprint.html"><strong aria-hidden="true">5.5.</strong> Blueprint</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/environment_variables.html"><strong aria-hidden="true">5.6.</strong> Environment Variables</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/configuration.html"><strong aria-hidden="true">5.7.</strong> Configuration</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/json.html"><strong aria-hidden="true">5.8.</strong> JSON Output</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/glob_patterns.html"><strong aria-hidden="true">5.9.</strong> Glob Patterns</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/command_line.html"><strong aria-hidden="true">5.10.</strong> Command Line</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="reference/limitations.html"><strong aria-hidden="true">5.11.</strong> Known Limitations</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/commands.html"><strong aria-hidden="true">6.</strong> Commands</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/new.html"><strong aria-hidden="true">6.1.</strong> orbit new</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/init.html"><strong aria-hidden="true">6.2.</strong> orbit init</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/info.html"><strong aria-hidden="true">6.3.</strong> orbit info</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/read.html"><strong aria-hidden="true">6.4.</strong> orbit read</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/get.html"><strong aria-hidden="true">6.5.</strong> orbit get</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/tree.html"><strong aria-hidden="true">6.6.</strong> orbit tree</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/lock.html"><strong aria-hidden="true">6.7.</strong> orbit lock</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/test.html"><strong aria-hidden="true">6.8.</strong> orbit test</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/build.html"><strong aria-hidden="true">6.9.</strong> orbit build</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/doc.html"><strong aria-hidden="true">6.10.</strong> orbit doc</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/publish.html"><strong aria-hidden="true">6.11.</strong> orbit publish</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/search.html"><strong aria-hidden="true">6.12.</strong> orbit search</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/install.html"><strong aria-hidden="true">6.13.</strong> orbit install</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/remove.html"><strong aria-hidden="true">6.14.</strong> orbit remove</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/env.html"><strong aria-hidden="true">6.15.</strong> orbit env</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="commands/config.html"><strong aria-hidden="true">6.16.</strong> orbit config</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="glossary.html"><strong aria-hidden="true">7.</strong> Appendix: Glossary</a></span></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            if (link.href === current_page
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                const clientRect = e.target.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                sessionStorage.setItem('sidebar-scroll-offset', clientRect.top - sidebarRect.top);
            }
        }, { passive: true });
        const sidebarScrollOffset = sessionStorage.getItem('sidebar-scroll-offset');
        sessionStorage.removeItem('sidebar-scroll-offset');
        if (sidebarScrollOffset !== null) {
            // preserve sidebar scroll position when navigating via links within sidebar
            const activeSection = this.querySelector('.active');
            if (activeSection) {
                const clientRect = activeSection.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                const currentOffset = clientRect.top - sidebarRect.top;
                this.scrollTop += currentOffset - parseFloat(sidebarScrollOffset);
            }
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        const sidebarAnchorToggles = document.querySelectorAll('.chapter-fold-toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(el => {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);

