use crate::event_context::EventContext;
use crate::action::on_keypress::OnKeypress;
use crate::util::{self, Operation};
use super::on_keypress::FnOnKeypress;

#[allow(dead_code)]
impl OnKeypress {
    pub fn spawn(process: &str) -> Box<dyn FnOnKeypress> {
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

    pub fn kill_process() -> Box<dyn FnOnKeypress> {
        Box::new(move |ctx: EventContext| {
            xcb::destroy_window(&ctx.conn, ctx.active_window()?);
            Ok(())
        })
    }

    pub fn toggle_fullscreen() -> Box<dyn FnOnKeypress> {
        Box::new(move |ctx: EventContext| {
            _ = ctx.clients.lock().unwrap()
                .set_fullscreen(0, Operation::Toggle)
                .map_err(|e| util::notify_error(e));

            Ok(())
        })
    }

    pub fn toggle_maximized() -> Box<dyn FnOnKeypress> {
        Box::new(move |ctx: EventContext| {
            _ = ctx.clients.lock().unwrap()
                .set_maximized(0, Operation::Toggle)
                .map_err(|e| util::notify_error(e));

            Ok(())
        })
    }

    pub fn swap_master() -> Box<dyn FnOnKeypress> {
        Box::new(move |ctx: EventContext| {
            ctx.clients.lock().unwrap().swap_master();
            Ok(())
        })
    }
}
