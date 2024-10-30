local luadec = require "luadec"
-- src/zsh_module.lua
local zsh_module = {
  enable = false,     -- Master toggle for enabling the config
  prompt = "default", -- Custom prompt string
  aliases = {},       -- Custom aliases
}

-- Function to set for the Zsh module
function zsh_module.opts(user_options)
  -- Merge user with the default
  for key, value in pairs(user_options) do
    if zsh_module[key] ~= nil then
      zsh_module[key] = value
    end
  end
  -- turn to luadec helper func

  -- Automatically apply the configuration after setting
  if zsh_module.enable then
    luadec.packages("AUR", {
      "zsh",
    })
    print("Configuring Zsh with the following settings:")
    print("Prompt:", zsh_module.prompt)
    print("Aliases:")
    for alias, command in pairs(zsh_module.aliases) do
      print("  alias " .. alias .. " = " .. command)
    end
  else
    print("Zsh configuration is disabled.")
  end
end

return zsh_module
