use log::info;

use crate::{
    event::{EventContext, ClientMessage},
    client::{
        Client,
        ClientState,
    },
    util::{self, Operation},
    errors::Error,
};

pub fn on_destroy_notify(ctx: EventContext, e: &xcb::DestroyNotifyEvent) -> Result<(), Error> {
    let mut screen = ctx.screen.lock().unwrap();

    let tag = screen.get_focused_tag_mut()?;
    let tag_id = tag.id;

    // focus the master (first) client if any; otherwise, disable the focus.
    if tag.get_focused_client().is_ok_and(|c| c.id == e.window()) {
        match tag.get_first_client_when(|c| c.is_controlled()) {
            Ok(c) => _ = tag.set_focused_client(c.id),
            Err(_) => util::disable_input_focus(&ctx.conn),
        };
    }

    tag.unmanage_client(e.window());

    _ = screen.refresh_tag(tag_id);

    Ok(())
}

pub fn on_map_request(ctx: EventContext, e: &xcb::MapRequestEvent) -> Result<(), Error> {
    let mut screen = ctx.screen.lock().unwrap();

    // The tag represents on which tag we should manage the client.
    // Generally, the sticky tag is reserved for storing clients that must be kept on the
    // screen independently of the current tag.
    let tag = match util::window_has_type(&ctx.conn, e.window(), ctx.conn.WM_WINDOW_TYPE_DOCK()) {
        true => screen.sticky_tag_mut(),
        false => screen.get_focused_tag_mut()?,
    };
    
    xcb::map_window(&ctx.conn, e.window());
    // If the client has already been managed by WM, we only need to map.
    if tag.contains_client(e.window()) {
        return Ok(())
    }

    let client = Client::new(&ctx.conn, e.window());

    util::set_client_tag(&ctx.conn, client.id, tag.id);
    tag.manage_client(client);
    tag.set_focused_client_if(e.window(), |c| c.is_controlled());

    let tag_id = tag.id;
    _ = screen.refresh_tag(tag_id);

    Ok(())
}

pub fn on_configure_request(e: &xcb::ConfigureRequestEvent, ctx: EventContext) -> Result<(), Error> {
    let mut values: Vec<(u16, u32)> = Vec::new();
    let mut maybe_push = |mask: u16, value: u32| {
        if e.value_mask() & mask > 0 {
            values.push((mask, value));
        }
    };

    maybe_push(xcb::CONFIG_WINDOW_WIDTH as u16, e.width() as u32);
    maybe_push(xcb::CONFIG_WINDOW_HEIGHT as u16, e.height() as u32);
    maybe_push(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, e.border_width() as u32);
    maybe_push(xcb::CONFIG_WINDOW_SIBLING as u16, e.sibling() as u32);
    maybe_push(xcb::CONFIG_WINDOW_STACK_MODE as u16, e.stack_mode() as u32);

    // if util::window_has_type(&ctx.conn, e.window(), ctx.conn.WM_WINDOW_TYPE_DIALOG()) {
    //     let geometry = xcb::get_geometry(&ctx.conn, e.window()).get_reply().unwrap();
    //     let screen = util::get_screen(&ctx.conn);
    //
    //     let x = (screen.width_in_pixels() - geometry.width()) / 2;
    //     let y = (screen.height_in_pixels() - geometry.height()) / 2;
    //
    //     maybe_push(xcb::CONFIG_WINDOW_X as u16, x as u32);
    //     maybe_push(xcb::CONFIG_WINDOW_Y as u16, y as u32);
    // } else {
    //     maybe_push(xcb::CONFIG_WINDOW_X as u16, e.x() as u32);
    //     maybe_push(xcb::CONFIG_WINDOW_Y as u16, e.y() as u32);
    // }

    xcb::configure_window(&ctx.conn, e.window(), &values);

    Ok(())
}

pub fn on_client_message(e: &xcb::ClientMessageEvent, ctx: EventContext) -> Result<(), Error> {
    let r#type = ClientMessage::from_atom(&ctx.conn, e.type_());
    let data = e.data().data32();

    log::trace!("Client message received: {}", r#type);

    let mut screen = ctx.screen.lock().unwrap();

    // TODO: Refactor into dedicated functions.
    match r#type {
        ClientMessage::ViewDesktop => {
            _ = screen.view_tag(data[0])?;
        },
        ClientMessage::ChangeState => {
            let action = Operation::from(data[0]);
            let state = data[1];

            if let Ok(t) = screen.get_focused_tag_mut() {
                let t_id = t.id;

                if let Ok(c) = t.get_client_mut(e.window()) {
                    if state == ctx.conn.WM_STATE_FULLSCREEN() {
                        log::info!("STATE FULLSCREEN");
                        _ = c.set_state(&ctx.conn, ClientState::Fullscreen, action);
                        _ = screen.refresh_tag(t_id);
                    }
                }
            }
        },
        ClientMessage::NotSupported => {
            log::warn!("Unsupported client message received. Atom={}", e.type_());
        },
    };

    Ok(())
}
