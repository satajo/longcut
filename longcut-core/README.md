# longcut-core

Longcut core logic and behaviour

## Configuration

The module is configured under the `core` key.

For the possible configurable values, see the [ConfigSchema definition in source](src/config.rs).

## Keys

A shortcut is a key with optional modifiers:

```yaml
shortcut: a
shortcut: Escape
shortcut:
  key: Up
  modifiers: [Shift, Control]
```

A single character stands for the key that types it. Anything else is an xkb keysym name, spelled
exactly as `xev`, `wev` or `xkbcli how-to-type` print it: `Escape`, `Return`, `BackSpace`,
`Caps_Lock`, `Page_Up`, `F1`, `XF86AudioMute`. Names are case-sensitive, so `escape` is not a key.
Every spelling xkb accepts is accepted; `Page_Up` and `Prior` name the same key.

The modifiers are `Shift`, `Control`, `Alt` and `Super`.
