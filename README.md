<div align="center">
	<h1>SapphireWM</h1>
	<p>
		<strong>A minimal, lightweight, and extendable tile window manager written in Rust</strong>
	</p>
</div>


<h2 align="center">Customization</h1>

One of the goals of Sapphire is to be as customizable as possible. Similar to [awesome](https://awesomewm.org/) and [dwm](https://dwm.suckless.org/), Sapphire allows you to customize the behavior of the window manager by modifying the source code.

### Keyboard and Keybindings

You can use the `Keyboard` struct to control keyboard events; a useful instance resides under your `WindowManager` instance.

To add a simple keybinding event, you can do:
```rust
wm.keyboard.append_keybindings(&[
    Keybinding::new()
        .on(&[modkey], "1")
        .group("Tag")
        .description("View tag[1].")
        .execute(Box::new(|ctx: EventContext| {
            let mut screen = ctx.screen.lock().unwrap();
            screen.view_tag(id)
        })),
]);
```

### Mouse

The `Mouse` struct allows you to control globally events triggered by the mouse, such as clicks and entering on clients.

To enable the focus on click feature you can do something like:
```rust
wm.mouse.on(MouseEvent::Click, Box::new(|ctx: EventContext, info: MouseInfo| {
    let mut screen = ctx.screen.lock().unwrap();

    let tag = screen.get_focused_tag_mut()?;
    let focus_id = tag.get_focused_client().map_or(0, |c| c.id);

    if focus_id != info.c_id {
        tag.set_focused_client_if(info.c_id, |c| c.is_controlled());
    }

    Ok(())
}));
```
