use crate::{
    event::EventContext,
    errors::Error,
};

pub trait FnOnKeypress: dyn_clone::DynClone {
    fn call(&self, ctx: EventContext) -> Result<(), Error>;
}

impl<F> FnOnKeypress for F
where 
    F: Fn(EventContext) -> Result<(), Error> + Clone 
{
    fn call(&self, ctx: EventContext) -> Result<(), Error> {
        self(ctx)
    }
}
