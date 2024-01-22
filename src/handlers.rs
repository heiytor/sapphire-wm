use xcb_util::ewmh;

use crate::{
    event_context::EventContext,
    clients::{
        client_type::ClientType,
        Client,
        client_action::ClientAction, client_state::ClientState,
    },
    util::{self, Operation},
    errors::Error,
};

pub fn on_map_request(e: &xcb::MapRequestEvent, ctx: EventContext) -> Result<(), Error> {
    xcb::map_window(&ctx.conn, e.window());

    let mut screen = ctx.screen.lock().unwrap();

    let mut r#type = ClientType::Normal;
    if util::window_has_type(&ctx.conn, e.window(), ctx.conn.WM_WINDOW_TYPE_DOCK()) {
        r#type = ClientType::Dock;
    }

    // The tag represents on which tag we should manage the client.
    // Generally, the sticky tag is reserved for storing clients that must be kept on the
    // screen independently of the current tag.
    let tag = match r#type {
        ClientType::Dock => screen.sticky_tag_mut(),
        _ => screen.get_focused_tag_mut().unwrap(), // TODO: remove this unwrap
    };
    let tag_id = tag.id;

    if tag.contains_client(e.window()) {
        return Ok(())
    }

    let mut client = Client::new(e.window());
    client.allow_action(&ctx.conn, ClientAction::Close);
    client.set_type(&ctx.conn, r#type, tag.id);

    // Retrieve some informations about the client
    if let Ok(pid) = ewmh::get_wm_pid(&ctx.conn, e.window()).get_reply() {
        client.wm_pid = Some(pid);
    }

    if let Ok(name) = ewmh::get_wm_name(&ctx.conn, e.window()).get_reply() {
        client.wm_class = Some(name.string().to_owned());
    }

    if let Ok(strut) = ewmh::get_wm_strut_partial(&ctx.conn, e.window()).get_reply() {
        client.padding.top = strut.top;
        client.padding.bottom = strut.bottom;
        client.padding.left = strut.left;
        client.padding.right = strut.right;
    };

    tag.manage_client(client);
    tag.set_focused_client_if(e.window(), |c| c.is_controlled());

    _ = screen.refresh_tag(tag_id);
    screen.refresh();

    Ok(())
}

pub fn on_destroy_notify(e: &xcb::DestroyNotifyEvent, ctx: EventContext) -> Result<(), Error> {
    let mut screen = ctx.screen.lock().unwrap();

    let tag = screen.get_focused_tag_mut().unwrap();
    let tag_id = tag.id;

    let client = match tag.get_client(e.window()) {
        Ok(c) => c.clone(), // TODO: is that clone really necessary?
        Err(_) => return Ok(()),
    };

    tag.unmanage_client(client.id);
    // TODO: destroy the window without PID. the PID must be the last thing that the WM uses to
    // destroy an window.
    if let Some(pid) = client.wm_pid {
        std::process::Command::new("kill").args(&["-9", &pid.to_string()]).output().unwrap();

        // Focus the master (first) client if any.
        if let Ok(c) = tag.get_first_client_when(|c| c.is_controlled()) {
            _ = tag.set_focused_client(c.id);
        }

        _ = screen.refresh_tag(tag_id);
        screen.refresh();
    }

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

    if util::window_has_type(&ctx.conn, e.window(), ctx.conn.WM_WINDOW_TYPE_DIALOG()) {
        let geometry = xcb::get_geometry(&ctx.conn, e.window()).get_reply().unwrap();
        let screen = util::get_screen(&ctx.conn);

        let x = (screen.width_in_pixels() - geometry.width()) / 2;
        let y = (screen.height_in_pixels() - geometry.height()) / 2;

        maybe_push(xcb::CONFIG_WINDOW_X as u16, x as u32);
        maybe_push(xcb::CONFIG_WINDOW_Y as u16, y as u32);
    } else {
        maybe_push(xcb::CONFIG_WINDOW_X as u16, e.x() as u32);
        maybe_push(xcb::CONFIG_WINDOW_Y as u16, e.y() as u32);
    }

    xcb::configure_window(&ctx.conn, e.window(), &values);

    Ok(())
}

pub fn on_client_message(e: &xcb::ClientMessageEvent, ctx: EventContext) -> Result<(), Error> {
    println!("client_message. atom: {}", e.type_());

    if e.type_() == ctx.conn.WM_STATE() {
        // SEE:
        // > https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142858672
        let data = e.data().data32();

        let state = data[1];
        let operation = match data[0] {
            ewmh::STATE_ADD => Operation::Add,
            ewmh::STATE_REMOVE => Operation::Remove,
            ewmh::STATE_TOGGLE => Operation::Toggle,
            _ => Operation::Unknown,
        };

        let mut screen = ctx.screen.lock().unwrap();

        if let Ok(t) = screen.get_focused_tag_mut() {
            let t_id = t.id;

            if let Ok(c) = t.get_client_mut(e.window()) {
                if state == ctx.conn.WM_STATE_FULLSCREEN() {
                    _ = c.set_state(&ctx.conn, ClientState::Fullscreen, operation);
                    _ = screen.refresh_tag(t_id);
                }
            }
        }
    }

    Ok(())
}
