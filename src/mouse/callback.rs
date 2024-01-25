use crate::{
    event::EventContext,
    client::ClientID,
    errors::Error,
};

pub trait FnOnClick: dyn_clone::DynClone {
    fn call(&self, ctx: EventContext, info: MouseInfo) -> Result<(), Error>;
}

impl<F> FnOnClick for F
where 
    F: Fn(EventContext, MouseInfo) -> Result<(), Error>  + Clone
{
    fn call(&self, ctx: EventContext, info: MouseInfo) -> Result<(), Error> {
        self(ctx, info)
    }
}

/// Represents information about mouse in events.
#[derive(Clone)]
pub struct MouseInfo {
    /// The client's ID where the mouse was pressed.
    pub c_id: ClientID,

    /// The x position of where the mouse was pressed. 0 is top-left.
    pub x: i16,

    /// The y position of where the mouse was pressed. 0 is top-left.
    pub y: i16,

    /// The mask of modifiers when the mouse was pressed. For example:
    /// ```
    /// // When pressing Mouse + Shift
    /// assert_eq!(modifier, 1);
    ///
    /// // When pressing Mouse + Shift + Ctrl
    /// assert_eq!(modifier, 1 | 4);
    /// ```
    ///
    /// You can also use `util::modkeys` to get the modifiers constants.
    pub modifier: u16,
}

impl MouseInfo {
    /// Creates a new `MouseInfo`. `Pos` is a tuple with (x, y) order.
    pub fn new(c_id: ClientID, modifier: u16, pos: (i16, i16)) -> Self {
        Self {
            c_id,
            x: pos.0,
            y: pos.1,
            modifier,
        }
    }
}
