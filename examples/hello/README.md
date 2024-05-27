# Hello project

This is an example project on how you can structure your CLI called `hello`
with a subcommand `world`.

To run it from this repo's root, make sure `sub` is installed and run:

```sh
./examples/hello/bin/hello
```

And to actually print "Hello, world!" run:

```sh
./examples/hello/bin/hello world
```

If you want to use this project as a template, simply replace the two
occurrences of `hello` with your CLI name (for example `hat`):

1. The file `bin/hello` should be renamed to `bin/hat`.
2. The argument `--name hello` in `bin/hello` (now `bin/hat`) should be changed
   to `--name hat`.
