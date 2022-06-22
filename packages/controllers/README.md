# SG Controllers: Common controllers for many contracts

This is a collection of "controllers" that we end up reimplementing in
many contracts. I use the word "controller" similar to the MVC framework
style, where it is an element that encapsulated business logic and data access.
We can also directly handle some `ExecuteMsg` and `QueryMsg` variants by
adding a sub-router to these controllers.

This work is derived from [cw-plus](https://github.com/CosmWasm/cw-plus/tree/main/packages/controllers).
