# longcut-x11

X11 client for Longcut: the launch keys, the session keyboard grab, key presses resolved
through the X server's own keymap, and active-window queries.

## Configuration

The module is configured under the `x11` key. Its `launcher` section holds the launch keys.

```yaml
x11:
  launcher:
    keys_launch_global:
      - Super_L
      - Super_R
    keys_launch_app:
      key: w
      modifiers: Super
```

`keys_launch_global` lists the keys that start a session in the global layers and `keys_launch_app`
the keys that start one in the layers of the focused application. At least one of the two is
required, and a key cannot be in both. Keys are spelled as described in the
[core key documentation](../longcut-core/README.md#keys), chords included.

Longcut binds the keys on the X server itself, so the window manager must not bind them as well.
From the moment a launch key is pressed every key goes to Longcut, including keys typed before
the panel is on screen. Caps Lock and Num Lock do not affect a launch key.

A launch key is only a launch key: to have it end the session as well, list it under the core
`keys_exit` too.

The keys held to press a launch key are not part of the session. A modifier held since the
launch is left out of the keys typed while it stays down, so with `Super_L` as the launch
key, `f` typed while Super is still held is `f`, not `Super+f`. Once released and pressed again, the
modifier counts as usual.
