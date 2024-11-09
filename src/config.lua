local programs = require("src.modules.programs")

local hello_world = require("src.hello_world")

hello_world.print_message() --                           :D

-- Example / experimentation

programs.zsh.opts({
  enable = true,
  prompt = "woah",
  aliases = {
    Ll = "ls -lah --color=auto",
    gs = "git status",
  },
})

programs.notzsh.opts({
  enable = true,
  prompt = "%B%F{red}%n@%m%f%b:%~%# ",
  aliases = {
    Ll = "ls -lah --color=auto",
    gs = "git status",
  },
})

local luadec = require 'luadec'
luadec.setup()


-- luadec.create_symlink("/home/main/dev/luadec/src/config.lua", "/home/main/.config/luadec/config.lua")

-- Does not install yet ()
luadec.packages("AUR", {
  "neovim",
  "tmux",
  "bat"
})

luadec.packages("Debian", {
  "neovim",
  "tmux",
  "hat",
})

-- Symlinks if content is path else copy / writes with
luadec.file(
  "/home/main/woahhie.sh", {
    vars = {
      x = "woah",
      y = "woah",
    },

    content = -- Or use a path /home/main
    [[
${x}
what isgoing on
what isgoing on
what isgoing on
what isgoing on
]]
  }
)
--local x = luadec.dir()

luadec.add_manager(
  {
    name = "PACMAN",
    add = "pacman -S #:?",
    remove = "pacman -R #:?",
    sync = "<COMMAND>",
    upgrade = "<COMMAND>",
  }
)

luadec.add_manager(
  {
    name = "AUR",
    add = 'printf "I just ran printf #:?\n"',
    remove = "<COMMAND> #:?",
    sync = "<COMMAND>",
    upgrade = "<COMMAND>",
  })

luadec.add_manager(
  {
    name = "Debian",
    add = "sudo apt install #:?",
    remove = "<COMMAND> #:?",
    sync = "<COMMAND>",
    upgrade = "<COMMAND>",
  })
