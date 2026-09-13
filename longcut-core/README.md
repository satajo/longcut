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

A press is the keysym it produces, with the held modifiers that took no part in producing it. Shift
is used up selecting a key's shifted level: Shift+1 on a US layout is `exclam`, Shift+a is `A` and
Shift+Tab is `ISO_Left_Tab`, none of them with `Shift`. A key with a single level has no use for
Shift, so Shift+F1 is `F1` with `Shift`. Control, Alt and Super are used up only where the keymap
gives a key a level for them, which is rare: Control+a is `a` with `Control`, while Control+Pause is
`Break`. A shortcut matches a press exactly, so `a` does not match Control+a.

Caps Lock never affects a shortcut: `a` matches with Caps Lock on. Text typed into a `text`
parameter with Caps Lock on is therefore lowercase.
