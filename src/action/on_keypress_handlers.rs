use crate::event_context::EventContext;
use crate::action::on_keypress::OnKeypress;
use crate::util::{self, Operation};
use super::on_keypress::FnOnKeypress;

#[allow(dead_code)]
impl OnKeypress {
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
