docker run -ti --rm \
	-v $PWD:/home/aghost-7/jira-releaser \
	-w /home/aghost-7/jira-releaser \
	-e RUST_SRC_PATH=/home/aghost-7/jira-releaser \
	-v $SSH_AUTH_SOCK:$SSH_AUTH_SOCK \
	-e SSH_AUTH_SOCK=$SSH_AUTH_SOCK \
	-v $HOME/.gitconfig:/home/aghost-7/.gitconfig \
	-e DISPLAY=$DISPLAY \
	-v /tmp/.X11-unix:/tmp/.X11-unix:ro \
	aghost7/rust-dev:stable \
	bash
