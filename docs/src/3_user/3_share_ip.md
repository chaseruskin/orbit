# Share IP

## Preparing for launch

1. Perform a dry run of the release procedure:
```
$ orbit launch --next <version>
```

2. Confirm releasing a new version:
```
$ orbit launch --next <version> --ready
```

> __Note:__ By default, a successful `orbit launch` will also automatically install that new version into your orbit cache.