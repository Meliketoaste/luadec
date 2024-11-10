> NEWS: Luadec WORKS! Still experimenting and it is subject to change.

# What is luadec?
Luadec configures your system declaratively with lua.

Luadec can be thought of as an simple library or API that lets you configure your system. Creating files, symlinks, and managing packages / services. And more eventually.

Lua has an nice module system. So you can structure it however you want. But I'm trying to imitate the experience of NixOS.

## Installing
```bash
git clone https://github.com/Meliketoaste/luadec.git
cd luadec
cargo install --path .

luadec # if not found add .cargo/bin/ to PATH
```

It will create an empty file at `/home/user/.config/luadec/config.lua`


## Usage
[This](https://github.com/Meliketoaste/luadec/blob/master/src/config.lua) counts as an example as well. Though its me just experimenting and debug.

Here is some actual example.
```lua
local luadec = require 'luadec'

-- Add package_manager
-- could technically be any command. #:? gets substituted with the packages
luadec.add_manager({
  name = "Debian",
  add = "sudo apt install #:?",
  remove = "<COMMAND> #:?",   -- Does not do anything yet
  sync = "<COMMAND>",         -- Does not do anything yet
  upgrade = "<COMMAND>",      -- Does not do anything yet
})

luadec.packages("Debian", {
  "neovim",
  "firefox",
})

luadec.file("/home/main/.xinitrc", {
    vars = {
      xinit_program = "i3",
    },
    content = "exec ${xinit_program}" -- Or use a path to create symlink or
    -- [[
    -- multi
    -- line
    --]]
  }
)
```

Then run `luadec` to apply it.

### Why lua?
1. I like lua.
2. I wanted to use Rust. And the crate [mlua](https://github.com/mlua-rs/mlua) is a nice way integrating it with lua.
3. Lua has a powerful module system which makes it great for extending functionality and creating modules and abstractions.

## similar / inspiration
- [Nix/NixOS](https://github.com/NixOS): for making me love declarative configuration
- [Rebos](https://gitlab.com/Oglo12/rebos): for the idea that we can support all package managers
- [consfigurator](https://github.com/spwhitton/consfigurator): Surprisingly similar but using lisp/Debian.
- [propellor](https://propellor.branchable.com)
- [declarix](https://github.com/stellatic/declarix)
- [config-king](https://github.com/kingdomkind/config-king)
- [pacdef](https://github.com/steven-omaha/pacdef)
- [system-manager](https://github.com/numtide/system-manager)
