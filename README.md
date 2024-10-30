> Luadec does not work yet, and is an work in progress.

Luadec tries to configure your system declaratively with lua.

It's essentially an API/Library to do system stuff. `require("luadec")` which lets us do symlinks, install packages, probably more.

Lua has an nice module system. So you can structure it however you want. But I'm trying to imitate the experience of NixOS.

The module system can let us create nice abstractions for configuring our system.

You should be able to bring any package manager (if your systems supports it) if you add something like this (credit: https://gitlab.com/Oglo12/rebos) but probably in lua:
```toml
add = "<COMMAND> #:?" # Example for APT: 'sudo apt install #:?'
remove = "<COMMAND> #:?" # Example for APT: 'sudo apt remove #:?'
sync = "<COMMAND>" # Example for APT: 'sudo apt update'
upgrade = "<COMMAND>" # Example for APT: 'sudo apt upgrade'

plural_name = "<PLURAL NAME>" # Example for APT: 'system packages'

hook_name = "<HOOK NAME>" # Example for APT: 'system_packages' (This must be filename safe!)
```
