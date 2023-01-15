# Plugins

`/dev/null` works on plugins. The app itself, is very dump, _you_ are the one who makes it intelligent.

In order to allow develoeprs to work with their own use-cases, `/dev/null` provides three plugins:

- [Static Content](./plugin/static-response.md)
- [Lua](./plugin/lua.md)
- [WASM](./plugin/wasm.md)

## Static Content

This is the simplest, and requires no coding on your part.

Fill out a config file, and `/dev/null` will match the incoming HTTP request with a known response, and respond! It's that easy.

## Lua

Do you need more control? Then Lua based config will be an option. The Lua script will get every request (assuming nothing else responded to it) and will be able to decide if it wants to respond. If it does, it will be able to fill out the response metadata.

## WASM

Like Lua, but on the WASM platform, allowing for libraries to be loaded and many different languages to be used.
