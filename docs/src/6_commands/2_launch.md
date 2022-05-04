# __orbit launch__

## __NAME__

launch - release an ip's next version 

## __SYNOPSIS__

```
orbit launch [options] <version>
```

## __DESCRIPTION__

This command will tag the current commit as a new version. By default, it
will perform a dry run of the launch process to verify the procedure will 
run with no errors. To actually perform a launch, include the '--ready'
flag.   

## __OPTIONS__

`--ready`  
      perform a real run through the launch process

## __EXAMPLES__

```
orbit launch 1.0.0
orbit launch 1.0.0 --ready
```