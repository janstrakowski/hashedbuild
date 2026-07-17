# Hashed Build
A more flexible alternative to Nix.

**This project is still in its early development - only one basic example works.**
## What I'm building?
A Nix-killer :)

HashedBuild is a reproducible build system powered by a flexible functional DSL. You give it a directory with your resources and HashedBuild code, and it processes the resources in a deterministic way
(same input gives the same output - also on your machine), outputing you the finished product. For example you give it a bunch of URLs to some Linux packages (e.g. coreutils, gcc) and a HashedBuild script describing
which tools pass the URLs through to get the results, and which tools pass the results through, etc., etc; in the end you get a rootfs of LFS to say.

It is also designed to be incremental. When an operation produces files, they are cached, indexing the copy by sha256 hash of the operation input. For example take look at a function `write_file` that takes a string and produces
a file contents of which is the string. You call the function by placing after it its argument, for example let us create a file that contains "Hello, World!":
```
write_file "Hello, World!"
```
When you save the code in `script.hashb` and call it:
```bash
# You must specify your cache directory
# This is where files created while executing the script are
# In this example it sets it to a directory "cache" in your current directory
# It will be created if it yet not exists
export HASHEDBUILD_CACHE=$(pwd)/cache
# This is the real command
hashedbuild-cli eval --source . --file script.hashb
```
The program prints on the terminal the absolute path to the produced file:
```
....../cache/McWJzkCzc1dcSzHj63epHDC_F3_kRxmOqnhdLgAZlSI
```
If you open it:
```
Hello, World
```
Your `Hello, World!` is there. If you change the `Hello, world!` into something else:
```
write_file "Something else..."
```
You get a different path:
......./cache/ngFwBbixJ4usaRnajJl_Lgn2NjDEYv1xCFUfK3tiK6Q
(the desired content is still there:)
```
Something else...
```
Let us construct the same string but in a different way:
```
write_file ("Something " "else...")
```
The path is exactly the same:
```
....../cache/ngFwBbixJ4usaRnajJl_Lgn2NjDEYv1xCFUfK3tiK6Q
```
(The result as well, but I will not show them this time)

No matter how you happend to get to the data, if it is the same, the path will stay the same: also, if the data is different the path is different.
Now let us imagine we in proccess of our bulding want to spin up a container that will have a full blown rootfs inside, will build a package for example bash, and returns the package directory with the files inside.
The plan for Hashbuild is to support containerized builds - it will be problably in the form of a builting function like `write_file' for example `build_in_container { image = lfs_chroot, package_source = source_file }`
(very oversimplified). So if we want to spin up a container, we can pass a rootfs that is a combination of some packages that we have already build before. If our LFS builds on the first run, great; but if we made
mistake the whole build process would be reset. But with Hashedbuild cache mechanism, we can just edit the part of code for building the package where we made the error and what gets rebuild is only the packages
that's input has not been changed by the result of the broken package. This is because if the input does not change, the output does not change as well.
### How it can be better than Nix?
Nix has a complex environment around it and in different fields does everything its own way. It has its own standard-environment which is driven by tangled bash scripts. The builds cannot use 
`#!/bin/bash` scripts without patching. Nixpkgs is a big central repository, which you can use only one snapshot of; and often there are packages only bound to the latest version (at the time of the snaphot).
I also plan to run the containerized builds in a standarised Open Container Intitiative conteiners and generaly I want to follow standards where possible.
## The Story
I used to use NixOs daily and got sick of some things. Not every package was there, and some didn't want to work. I had to debug what is going on, and I have written a couple of my own packages.

Then I learned its package developer side, stdenv, browsed in the nixpkgs source. I could not follow the nixpkgs source to fix my issues, and just wanted to code instead of defining a package for every not common thing I want, so
I finally gave up. I switch to WSL. I plan actually to return to Linux, when I finish my project, and use it instead of Nixos.

I thought for a long time of doing such thing myself, and in the begging of this Summer, I finally started writing some code. Apparently, I changed my mind completely a few times, and just
abadoned my old codebase and started a new in another language. I went C -> Zig -> Rust. This one is in Rust and I will stick with Rust because:
- it has a mature ecosystem,
- it guarantees memory safety,
- it has a lot of high level features that boost my work,
- the community is huge, and there are packages for almost everything I need.

So in this week I switched to Rust but yesterday (17th July) and today (18th July) worked all the day, really hard on the project; and I can run a working examples.

### Do I vibe-code?
Yes, but I can program myself. I treat it as a typing speed boost and design boost in simple matters.
