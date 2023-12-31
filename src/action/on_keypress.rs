use crate::event_context::EventContext;

pub trait FnOnKeypress: dyn_clone::DynClone {
    fn call(&self, ctx: EventContext) -> Result<(), String>;
}

impl<F> FnOnKeypress for F
where 
    F: Fn(EventContext) -> Result<(), String> + Clone 
{
    fn call(&self, ctx: EventContext) -> Result<(), String> {
        self(ctx)
    }
}

pub struct OnKeypress {
    callback: Box<dyn FnOnKeypress>,

    pub modkey: Vec<u16>,
    pub ch: char,
}

impl OnKeypress {
    pub fn new(
        modkey: &[u16],
        ch: char,
        callback: Box<dyn FnOnKeypress>,
    ) -> Self {
        OnKeypress { 
            modkey: modkey.to_vec(),
            ch,
            callback,
        }
    }
}

impl Clone for OnKeypress {
    fn clone(&self) -> Self {
        Self {
            ch: self.ch.clone(),
            modkey: self.modkey.clone(),
            callback: dyn_clone::clone_box(&*self.callback),
        }
    }
}

impl OnKeypress {
    #[inline]
    pub fn call(&self, ctx: EventContext) -> Result<(), String> {
        self.callback.call(ctx)
    }
}
