# 介绍

`Async-graphql`是用Rust语言实现的GraphQL服务端库。它完全兼容GraphQL规范以及绝大部分的扩展功能，类型安全并且高性能。

你可以用Rust语言的方式来定义Schema，过程宏会自动生成GraphQL查询的框架代码，没有扩展Rust的语法，意味着Rustfmt可以正常使用，我很看重这一点，这也是为什么我会开发`Async-graphql`的原因之一。

## 为什么我要开发Async-graphql？

我喜欢GraphQL和Rust，之前我一直用`Juniper`，它解决了我用Rust实现GraphQL服务器的问题，但也有一些遗憾，其中最重要的是它当时不支持async/await，所以我决定做一个给自己用。
