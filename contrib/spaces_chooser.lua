-- spaces_chooser.lua - a Hammerspoon chooser to jump to a named Mission Control space.
--
-- This module shells out to the `move_window list_spaces --json` subcommand,
-- builds an hs.chooser from the result and, on selection, jumps to the chosen
-- space via hs.spaces.gotoSpace.
--
-- INTEGRATION
-- -----------
-- 1. Drop this file next to your init.lua, e.g. ~/.hammerspoon/spaces_chooser.lua.
--
-- 2. Load it from ~/.hammerspoon/init.lua:
--
--        local spacesChooser = require("spaces_chooser")
--
--    If your move_window binary is not at ~/bin/move_window, set the path:
--
--        spacesChooser.binary = "/usr/local/bin/move_window"
--
-- 3a. Plain hotkey example (Ctrl + Alt + S opens the chooser):
--
--        hs.hotkey.bind({ "ctrl", "alt" }, "s", function()
--            spacesChooser.show()
--        end)
--
-- 3b. Or bind it into your existing hyperM modal. You already have
--     `local hyperM = hs.hotkey.modal.new({ "ctrl" }, "M")` with bindings such
--     as `hyperM:bind("", "f", function() ... hyperM:exit() end)`. Add:
--
--        hyperM:bind("", "s", function()
--            spacesChooser.show()
--            hyperM:exit()
--        end)
--
-- Reloading your Hammerspoon config after these edits is enough; this file does
-- not modify any of your dotfiles itself.

local M = {}

-- Path to the move_window binary. Override from init.lua if needed.
M.binary = os.getenv("HOME") .. "/bin/move_window"

-- Reads the spaces listing from move_window and returns the decoded table, or
-- nil plus an error message on failure.
local function readSpaces()
	local command = M.binary .. " list_spaces --json"
	local out, status, _, _ = hs.execute(command, false)
	if not status or out == nil or out == "" then
		return nil, "move_window list_spaces failed (is " .. M.binary .. " installed?)"
	end
	local ok, decoded = pcall(hs.json.decode, out)
	if not ok or type(decoded) ~= "table" then
		return nil, "could not decode list_spaces JSON"
	end
	return decoded, nil
end

-- Turns the decoded displays into hs.chooser choices in Mission Control order.
local function buildChoices(displays)
	local choices = {}
	for _, display in ipairs(displays) do
		for _, space in ipairs(display.spaces) do
			local label
			if space.index ~= nil then
				label = "Space " .. tostring(space.index) .. " on " .. display.display
			else
				label = space.type .. " space on " .. display.display
			end
			local subText = label .. "  (id " .. tostring(space.id) .. ")"
			if space.current then
				subText = subText .. "  [current]"
			end
			table.insert(choices, {
				text = space.name,
				subText = subText,
				spaceId = space.id,
			})
		end
	end
	return choices
end

-- Builds and shows the chooser. On selection, jumps to the picked space.
function M.show()
	local displays, err = readSpaces()
	if not displays then
		hs.alert.show(err)
		return
	end

	local choices = buildChoices(displays)
	if #choices == 0 then
		hs.alert.show("No spaces found")
		return
	end

	local chooser = hs.chooser.new(function(choice)
		if not choice then
			return
		end
		local ok, result = pcall(hs.spaces.gotoSpace, choice.spaceId)
		if not ok or result == false then
			hs.alert.show("Failed to switch to " .. choice.text)
		end
	end)

	chooser:placeholderText("Jump to space")
	chooser:choices(choices)
	chooser:show()
end

return M
