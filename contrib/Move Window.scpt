on call_move_window(cmdline)
	tell Application "LaunchBar"
		hide
	end tell
	do shell script ("$SHELL -c 'move_window " & cmdline & "'")
end call_move_window

on handle_string(given_string)
	call_move_window(given_string)
end handle_string

