# Flat Collections

A library that provides lightweight and memory-efficient associative data structures.

### Asymptotics:

| operation | average | worst   | best    |
|:----------|:--------|:--------|:--------|
| lookup    | O(logn) | O(logn) | O(logn) |
| insert    | O(n)    | O(n)    | O(1)    |
| remove    | O(n)    | O(n)    | O(1)    |

Insert and remove work in O(1) when dealing with last element.

### Types:
- `FlatMap` - mutable map, backed by `Vec`
- `FlatSet` - mutable set, backed by `FlatMap`
