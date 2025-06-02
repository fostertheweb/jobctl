# eval in ~/.zshrc

function register_job() {
	if [[ $? -eq 146 ]]; then
		PID=$(jobs -l | tail -n 1 | awk '{print $3}')
		cargo run --bin jobctl -- register $PID
	fi
}

autoload -Uz add-zsh-hook
add-zsh-hook precmd register_job
