use crate::client::{Dir, ClientState};
use crate::event_context::EventContext;
use crate::action::on_keypress::OnKeypressAction;
use crate::util;

#[allow(dead_code)]
impl OnKeypressAction {
    pub fn spawn(process: &str) -> Box<dyn Fn(EventContext) -> Result<(), String>> {
        let process_parts: Vec<&str> = process.split_whitespace().collect();

        match process_parts.split_first() {
            Some((command, args)) => {
                let command = command.to_string();
                let args: Vec<String> = args.iter().map(|&s| s.to_string()).collect();

                Box::new(move |_| {
                    std::process::Command::new(&command)
                        .args(&args)
                        .spawn()
                        .map_err(|e| e.to_string())?;

                    Ok(())
                })
            },
            None => {
                Box::new(move |_| Err("Invalid process string".to_string()))
            },
        }
    }

    pub fn kill_process() -> Box<dyn Fn(EventContext) -> Result<(), String>> {
        Box::new(move |ctx| {
            xcb::destroy_window(&ctx.conn, ctx.active_window()?);
            Ok(())
        })
    }

    pub fn focus_left() -> Box<dyn Fn(EventContext) -> Result<(), String>> {
        Box::new(move |ctx| {
            ctx.clients.lock().unwrap().move_focus(Dir::Left);
            Ok(())
        })
    }

    pub fn focus_right() -> Box<dyn Fn(EventContext) -> Result<(), String>> {
        Box::new(move |ctx| {
            ctx.clients.lock().unwrap().move_focus(Dir::Right);
            Ok(())
        })
    }

    pub fn toggle_fullscreen() -> Box<dyn Fn(EventContext) -> Result<(), String>> {
        Box::new(move |ctx| {
            _ = ctx.clients.lock().unwrap()
                .set_fullscreen(0, ClientState::Toggle)
                .map_err(|e| util::notify_error(e));

            Ok(())
        })
    }

    pub fn swap_master() -> Box<dyn Fn(EventContext) -> Result<(), String>> {
        Box::new(move |ctx| {
            ctx.clients.lock().unwrap().swap_master();
            Ok(())
        })
    }
}
