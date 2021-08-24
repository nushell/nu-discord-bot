# Nushell-API

A Rocket server that exposes an endpoint to run sandboxed Nushell commands.

## To use

Run `cargo run` and then send a post command as follows:

```sh
curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"input":"config"}' \
  http://localhost:8000/
```
