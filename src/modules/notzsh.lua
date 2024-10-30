-- luadec = require "luadec"
-- src/zsh_module.lua
local nozsh = {
  enable = false,     -- Master toggle for enabling the config
  prompt = "default", -- Custom prompt string
  aliases = {},       -- Custom aliases
}

-- Function to set for the Zsh module
function nozsh.opts(user_options)
  -- Merge user with the default
  for key, value in pairs(user_options) do
    if nozsh[key] ~= nil then
      nozsh[key] = value
    end
  end
  -- turn to luadec helper func

  -- Automatically apply the configuration after setting
  if nozsh.enable then
    print("Configuring Zsh with the following settings:")
    print("Prompt:", nozsh.prompt)
    print("Aliases:")
    for alias, command in pairs(nozsh.aliases) do
      print("  alias " .. alias .. " = " .. command)
    end
  else
    print("Zsh configuration is disabled.")
  end
end

return nozsh
