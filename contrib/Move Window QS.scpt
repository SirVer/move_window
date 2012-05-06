using terms from application "Quicksilver"
	on process text t
		tell application "Quicksilver" to close
		do shell script ("/usr/local/bin/move_window " & t)
		return nothing
	end process text
end using terms from

