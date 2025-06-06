# eval in ~/.zshrc

function register_job() {
	if [[ $? -eq 146 ]]; then
		JOB=$(jobs -l | tail -n 1)
		JOB_NUMBER=$(echo $JOB | awk -F'[][]' '{ print $2 }')
		PID=$(echo $JOB | cut -d']' -f2 | awk '{ print $2 }')
		cargo run --bin jobctl -- register --pid $PID --number $JOB_NUMBER
	fi
}

autoload -Uz add-zsh-hook
add-zsh-hook precmd register_job
