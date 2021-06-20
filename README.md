# cmd-tunnel

This is a small utility that tunnels commands between terminals and forwards
stdout and stderr. I wrote this because I was tired of the Windows terminal
not supporting the affordances I'd become used to in Linux land with zsh. This
lets me be in zsh but run commands and such in a Windows terminal.

# Build

1. Install Rust from [rustup](https://rustup.rs/)
2. Clone this repo and CD into it
3. Run `cargo install --path .`

Now you should have 2 binaries in your `~/.cargo/bin` path called `cmd-tunnel-server` and `cmd-tunnel-client`.

# Run

Open the terminal where you want commands to be run which in my case would be a Windows terminal with the specific environment setup that lets me run the commands that I need run and then run `cmd-tunnel-server`.

Next open up the client terminal from where you'll be issuing commands which in my case would be a [WSL](https://en.wikipedia.org/wiki/Windows_Subsystem_for_Linux) terminal with `zsh` and all that goodness and start running commands like so:

```bash
cmd-tunnel-client ctest -C Debug -R bsi_local_ut
```

I would probably alias `cmd-tunnel-client` to `cx` or something so its easier on my fingers. That's it.