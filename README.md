# blur_plus (Horizon native mod)

A 1:1 Rust port of Motschen's Blur (MIT, (c) 2017 tterrag): blurred screens
with a gradient background. The package is the `.hmod` source: a native mod
built against the pinned Horizon SDK (`api 0.2`, game 26.3).

## Behavior (the original's, kept)

- The 1:1 logic core lives in `runtime.rs`, untouched: the config with its
  v3 migration, the HSB and #RRGGBB color math, the ten easing curves, the
  blur-radius, background-alpha and gradient animation handlers, the
  render-pass flow (onRender / renderBackground / onRenderEnd), the
  per-screen gates from every upstream mixin, the options coefficient and
  the debug HUD renderer.
- The settings are the original's config fields, held on Horizon's
  Options > Mods page, same defaults, labels in `assets/blur_plus/lang/`.
  The two screen lists (force-enabled, force-disabled) stay compiled-in:
  the pinned API has no string-list setting kind.

## Wiring

- register: the original's config fields registered as settings, same
  defaults.
- The settings read back as the runtime's config through the host's typed
  readers (bool, choice, f64, color), so the animations run from the
  settings the moment a screen hook lands.

## Horizon seams

- The blurred draw waits on [horizon-rust/horizon#19349](https://github.com/horizon-rust/horizon/issues/19349):
  the pinned mod API has no screen-render hook a mod can use to draw a
  blurred background over the world, and no screen-open hook for the gates
  to consult. Until one lands the core compiles, its animations run from
  the settings, and the screens draw unblurred.

## License

MIT, upstream (c) 2017 tterrag.
