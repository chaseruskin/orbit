# Make Vendors

## Registering a Vendor

1. Clone the repository to a known location:

```
$ git clone <repository> "$(orbit env ORBIT_HOME)/vendor/<name>"
```

2. Link the vendor's configuration file to your vendor index:

```
$ orbit config --global --append vendor.index="$(orbit env ORBIT_HOME)/vendor/<name>/index.toml"
```