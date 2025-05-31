# eval in ~/.bashrc:

# associative array (bash 4+) to remember which JIDs we’ve already logged
declare -A _SUSP_LOGGED

# seed on every prompt
precmd() {
	_SUSP_LOGGED=()
	while read -r line; do
		[[ $line == *Stopped* ]] || continue
		# [1]+ 12345 Stopped ... → jid=1
		jid=${line#\[}
		jid=${jid%%]*}
		_SUSP_LOGGED[$jid]=1
	done < <(jobs -l)
}

# on any child-status change, find new “Stopped” jobs
trap '
  while read -r line; do
    [[ $line == *Stopped* ]] || continue
    jid=${line#\[}; jid=${jid%%]*}
    if [[ -z ${_SUSP_LOGGED[$jid]} ]]; then
      echo "🔔 Job #$jid suspended at $(date)" >> ~/.suspend.log
      _SUSP_LOGGED[$jid]=1
    fi
  done < <(jobs -l)
' SIGCHLD
