# eval in ~/.zshrc

# associative array to track which PIDs are already “known stopped”
typeset -A _SUSPENDED_JOBS

# initialize map of stopped jobs
precmd() {
	_SUSPENDED_JOBS=()
	while read -r line; do
		[[ $line == *Stopped* ]] || continue
		# jobs -l prints: [1]  + 12345 Stopped ...
		local jid=$(print $line | awk '{print $1}' | tr -d '[]')
		_SUSPENDED_JOBS[$jid]=1
	done < <(jobs -l)
}

TRAPCHLD() {
	# on any child status change, re-scan for newly-stopped jobs
	while read -r line; do
		[[ $line == *Stopped* ]] || continue
		local jid=$(print $line | awk '{print $1}' | tr -d '[]')
		if [[ -z ${_SUSPENDED_JOBS[$jid]} ]]; then
			# Found a freshly‐suspended job
			echo "🔔 Job #$jid suspended at $(date)" >>~/.suspend.log
			# ... you can insert any other hook/action here ...
			_SUSPENDED_JOBS[$jid]=1
		fi
	done < <(jobs -l)
}
