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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="introduction.html"><strong aria-hidden="true">1.</strong> 介绍</a></li><li class="chapter-item expanded "><a href="quickstart.html"><strong aria-hidden="true">2.</strong> 快速开始</a></li><li class="chapter-item expanded "><a href="typesystem.html"><strong aria-hidden="true">3.</strong> 类型系统</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="define_simple_object.html"><strong aria-hidden="true">3.1.</strong> 简单对象 (SimpleObject)</a></li><li class="chapter-item expanded "><a href="define_complex_object.html"><strong aria-hidden="true">3.2.</strong> 对象 (Object)</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="context.html"><strong aria-hidden="true">3.2.1.</strong> 查询上下文 (Context)</a></li><li class="chapter-item expanded "><a href="error_handling.html"><strong aria-hidden="true">3.2.2.</strong> 错误处理</a></li><li class="chapter-item expanded "><a href="merging_objects.html"><strong aria-hidden="true">3.2.3.</strong> 合并对象 (MergedObject)</a></li><li class="chapter-item expanded "><a href="derived_fields.html"><strong aria-hidden="true">3.2.4.</strong> 派生字段</a></li></ol></li><li class="chapter-item expanded "><a href="define_enum.html"><strong aria-hidden="true">3.3.</strong> 枚举 (Enum)</a></li><li class="chapter-item expanded "><a href="define_interface.html"><strong aria-hidden="true">3.4.</strong> 接口 (Interface)</a></li><li class="chapter-item expanded "><a href="define_union.html"><strong aria-hidden="true">3.5.</strong> 联合 (Union)</a></li><li class="chapter-item expanded "><a href="define_input_object.html"><strong aria-hidden="true">3.6.</strong> 输入对象 (InputObject)</a></li><li class="chapter-item expanded "><a href="default_value.html"><strong aria-hidden="true">3.7.</strong> 默认值</a></li></ol></li><li class="chapter-item expanded "><a href="define_schema.html"><strong aria-hidden="true">4.</strong> 定义模式 (Schema)</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="query_and_mutation.html"><strong aria-hidden="true">4.1.</strong> 查询和变更</a></li><li class="chapter-item expanded "><a href="subscription.html"><strong aria-hidden="true">4.2.</strong> 订阅</a></li></ol></li><li class="chapter-item expanded "><a href="utilities.html"><strong aria-hidden="true">5.</strong> 实用功能</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="field_guard.html"><strong aria-hidden="true">5.1.</strong> 字段守卫</a></li><li class="chapter-item expanded "><a href="input_value_validators.html"><strong aria-hidden="true">5.2.</strong> 输入值校验器</a></li><li class="chapter-item expanded "><a href="cache_control.html"><strong aria-hidden="true">5.3.</strong> 查询缓存控制</a></li><li class="chapter-item expanded "><a href="cursor_connections.html"><strong aria-hidden="true">5.4.</strong> 游标连接</a></li><li class="chapter-item expanded "><a href="error_extensions.html"><strong aria-hidden="true">5.5.</strong> 错误扩展</a></li><li class="chapter-item expanded "><a href="apollo_tracing.html"><strong aria-hidden="true">5.6.</strong> Apollo Tracing 支持</a></li><li class="chapter-item expanded "><a href="depth_and_complexity.html"><strong aria-hidden="true">5.7.</strong> 查询的深度和复杂度</a></li><li class="chapter-item expanded "><a href="visibility.html"><strong aria-hidden="true">5.8.</strong> 在内省中隐藏内容</a></li></ol></li><li class="chapter-item expanded "><a href="extensions.html"><strong aria-hidden="true">6.</strong> 扩展</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="extensions_inner_working.html"><strong aria-hidden="true">6.1.</strong> 扩展如何工作</a></li><li class="chapter-item expanded "><a href="extensions_available.html"><strong aria-hidden="true">6.2.</strong> 可用的扩展列表</a></li></ol></li><li class="chapter-item expanded "><a href="integrations.html"><strong aria-hidden="true">7.</strong> 集成到 WebServer</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="integrations_to_poem.html"><strong aria-hidden="true">7.1.</strong> Poem</a></li><li class="chapter-item expanded "><a href="integrations_to_warp.html"><strong aria-hidden="true">7.2.</strong> Warp</a></li><li class="chapter-item expanded "><a href="integrations_to_actix_web.html"><strong aria-hidden="true">7.3.</strong> Actix-web</a></li></ol></li><li class="chapter-item expanded "><a href="advanced_topics.html"><strong aria-hidden="true">8.</strong> 高级主题</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="custom_scalars.html"><strong aria-hidden="true">8.1.</strong> 自定义标量</a></li><li class="chapter-item expanded "><a href="dataloader.html"><strong aria-hidden="true">8.2.</strong> 优化查询（解决 N+1 问题）</a></li><li class="chapter-item expanded "><a href="custom_directive.html"><strong aria-hidden="true">8.3.</strong> 自定义指令</a></li><li class="chapter-item expanded "><a href="apollo_federation.html"><strong aria-hidden="true">8.4.</strong> Apollo Federation 集成</a></li></ol></li></ol>';
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
