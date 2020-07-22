# 介绍

`Async-graphql`是用Rust语言实现的GraphQL服务端库。它完全兼容GraphQL规范以及绝大部分的扩展功能，类型安全并且高性能。

你可以用Rust语言的方式来定义Schema，过程宏会自动生成GraphQL查询的框架代码，没有扩展Rust的语法，意味着Rustfmt可以正常使用，我很看重这一点，这也是为什么我会开发`Async-graphql`的原因之一。

## 为什么我要开发Async-graphql？

我喜欢GraphQL和Rust，之前我一直用`Juniper`，它解决了我用Rust实现GraphQL服务器的问题，但也有一些遗憾，其中最重要的是它当时不支持async/await，所以我决定做一个给自己用。

## 稳定性

__这个项目目前不遵循 [Semantic Versioning (SemVer)](https://semver.org/) ，并且可能会在任何版本号上发生不向前兼容的变化。一旦项目达到`v2.0.0`，我们将确保每次更新符合SemVer的规范。__

尽管这个项目在`v1.0.0`之上，但是我们正在快速迭代和改进API。这导致了版本控制问题，这些问题不容易解决，因为这个项目很快就流行起来(2020年3月才开始开发)。

我们目前计划在`v2.0.0`发布之后开始执行SemVer，这将在API开始稳定之后发生。不幸的是，我们目前还没有这方面的时间表。

根据Rust关于“ 1.0.0”之前的版本的政策，我们将尝试将不兼容的变化限制为次要版本更改，但如果升级后似乎未通过编译，则可能需要深入研究编译器错误以更新某些已更改的语法。如果有些奇怪，请随时打开 [issue](https://github.com/async-graphql/async-graphql/issues) 。
