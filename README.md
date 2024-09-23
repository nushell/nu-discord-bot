# Nu-discord-bot

A Discord bot built with Serenity that runs Nushell commands in a custom context with limited privileges.

## Security

Even though the set of nushell commands made available to users of this bot is limited and excludes some of
the most obvious ways to interact with the host's file system and network, this is not, by itself, a secure level
of sandboxing. If you're going to use this bot on discord servers with untrusted users, you should run it inside
something like a docker container.

## To use

Load your bot discord token into the `DISCORD_TOKEN` environment variable. Run `cargo run`.

You should see you discord bot activate, and you can run commands with the prefix `nu!` e.g. `nu! help`.
