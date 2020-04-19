# 自定义扩展

一个GraphQL扩展对象能够接收一个查询执行各个阶段的事件，你可以收集想要的数据，这些数据能够在查询结果中返回。

你只需要实现`async_graphql::Extension`就能够定义一个扩展对象，然后在创建`Schema`的时候调用`Schema::extension`应用扩展。

你可以参考[Apollo tracing](https://github.com/sunli829/async-graphql/blob/master/src/extensions/tracing.rs)来实现自己的扩展类型。