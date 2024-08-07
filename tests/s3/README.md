# /s3

This directory hosts code for demonstrating the dynamic symbol transformation algorithm.

```
ip-c:0.1.0
└─ ip-a:0.1.0
   └─ ip-b:0.1.0
```

An entity `dupe` exists in both `ip-b` and `ip-c`. Since `ip-b` is an indirect dependncy to `ip-c`, dynamic symbol transformation will occur to resolve the namespace issue and allow both entities to co-exist.