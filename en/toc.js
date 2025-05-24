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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="introduction.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="quickstart.html"><strong aria-hidden="true">2.</strong> Quickstart</a></li><li class="chapter-item expanded "><a href="typesystem.html"><strong aria-hidden="true">3.</strong> Type System</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="define_simple_object.html"><strong aria-hidden="true">3.1.</strong> SimpleObject</a></li><li class="chapter-item expanded "><a href="define_complex_object.html"><strong aria-hidden="true">3.2.</strong> Object</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="context.html"><strong aria-hidden="true">3.2.1.</strong> Context</a></li><li class="chapter-item expanded "><a href="error_handling.html"><strong aria-hidden="true">3.2.2.</strong> Error handling</a></li><li class="chapter-item expanded "><a href="merging_objects.html"><strong aria-hidden="true">3.2.3.</strong> Merging Objects / Subscriptions</a></li><li class="chapter-item expanded "><a href="derived_fields.html"><strong aria-hidden="true">3.2.4.</strong> Derived fields</a></li></ol></li><li class="chapter-item expanded "><a href="define_enum.html"><strong aria-hidden="true">3.3.</strong> Enum</a></li><li class="chapter-item expanded "><a href="define_interface.html"><strong aria-hidden="true">3.4.</strong> Interface</a></li><li class="chapter-item expanded "><a href="define_union.html"><strong aria-hidden="true">3.5.</strong> Union</a></li><li class="chapter-item expanded "><a href="define_input_object.html"><strong aria-hidden="true">3.6.</strong> InputObject</a></li><li class="chapter-item expanded "><a href="define_one_of_object.html"><strong aria-hidden="true">3.7.</strong> OneofObject</a></li><li class="chapter-item expanded "><a href="default_value.html"><strong aria-hidden="true">3.8.</strong> Default value</a></li><li class="chapter-item expanded "><a href="generic.html"><strong aria-hidden="true">3.9.</strong> Generics</a></li></ol></li><li class="chapter-item expanded "><a href="define_schema.html"><strong aria-hidden="true">4.</strong> Schema</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="query_and_mutation.html"><strong aria-hidden="true">4.1.</strong> Query and Mutation</a></li><li class="chapter-item expanded "><a href="subscription.html"><strong aria-hidden="true">4.2.</strong> Subscription</a></li><li class="chapter-item expanded "><a href="sdl_export.html"><strong aria-hidden="true">4.3.</strong> SDL Export</a></li></ol></li><li class="chapter-item expanded "><a href="utilities.html"><strong aria-hidden="true">5.</strong> Utilities</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="field_guard.html"><strong aria-hidden="true">5.1.</strong> Field guard</a></li><li class="chapter-item expanded "><a href="input_value_validators.html"><strong aria-hidden="true">5.2.</strong> Input value validators</a></li><li class="chapter-item expanded "><a href="cache_control.html"><strong aria-hidden="true">5.3.</strong> Cache control</a></li><li class="chapter-item expanded "><a href="cursor_connections.html"><strong aria-hidden="true">5.4.</strong> Cursor connections</a></li><li class="chapter-item expanded "><a href="error_extensions.html"><strong aria-hidden="true">5.5.</strong> Error extensions</a></li><li class="chapter-item expanded "><a href="apollo_tracing.html"><strong aria-hidden="true">5.6.</strong> Apollo Tracing</a></li><li class="chapter-item expanded "><a href="depth_and_complexity.html"><strong aria-hidden="true">5.7.</strong> Query complexity and depth</a></li><li class="chapter-item expanded "><a href="visibility.html"><strong aria-hidden="true">5.8.</strong> Hide content in introspection</a></li></ol></li><li class="chapter-item expanded "><a href="extensions.html"><strong aria-hidden="true">6.</strong> Extensions</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="extensions_inner_working.html"><strong aria-hidden="true">6.1.</strong> How extensions are working</a></li><li class="chapter-item expanded "><a href="extensions_available.html"><strong aria-hidden="true">6.2.</strong> Available extensions</a></li></ol></li><li class="chapter-item expanded "><a href="integrations.html"><strong aria-hidden="true">7.</strong> Integrations</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="integrations_to_poem.html"><strong aria-hidden="true">7.1.</strong> Poem</a></li><li class="chapter-item expanded "><a href="integrations_to_warp.html"><strong aria-hidden="true">7.2.</strong> Warp</a></li><li class="chapter-item expanded "><a href="integrations_to_actix_web.html"><strong aria-hidden="true">7.3.</strong> Actix-web</a></li></ol></li><li class="chapter-item expanded "><a href="advanced_topics.html"><strong aria-hidden="true">8.</strong> Advanced topics</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="custom_scalars.html"><strong aria-hidden="true">8.1.</strong> Custom scalars</a></li><li class="chapter-item expanded "><a href="dataloader.html"><strong aria-hidden="true">8.2.</strong> Optimizing N+1 queries</a></li><li class="chapter-item expanded "><a href="custom_directive.html"><strong aria-hidden="true">8.3.</strong> Custom directive</a></li><li class="chapter-item expanded "><a href="apollo_federation.html"><strong aria-hidden="true">8.4.</strong> Apollo Federation</a></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
