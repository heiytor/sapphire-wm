use xcb_util::ewmh;

use crate::util::Operation;

#[derive(PartialEq, Debug)]
pub enum ClientState {
    Fullscreen,
    Maximized,
}

#[derive(PartialEq, Debug)]
pub enum ClientType {
    Dock,
    Normal,
}

pub struct Client {
    is_controlled: bool,
    is_visible: bool,
    r#type: ClientType,

    pub pid: u32,
    pub wid: u32,
    pub states: Vec<ClientState>,
    pub padding_top: u32,
    pub padding_bottom: u32,
    pub padding_left: u32,
    pub padding_right: u32,
    pub desktop: u32,
}

impl Default for Client {
    fn default() -> Self {
        Client { 
            pid: 0,
            wid: 0,
            is_controlled: false,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: 0,
            padding_right: 0,
            is_visible: false,
            r#type: ClientType::Normal,
            states: Vec::new(),
            desktop: 0,
        }
    }
}

impl Client {
    pub fn new(wid: u32) -> Self {
        Client { 
            wid,
            pid: 0,
            is_controlled: true,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: 0,
            padding_right: 0,
            is_visible: true,
            r#type: ClientType::Normal,
            states: Vec::new(),
            desktop: 0,
        }
    }
}

impl Client {
    /// Sets the padding values for the client.
    ///
    /// # Arguments
    ///
    /// * `top` - The padding value for the top side.
    /// * `bottom` - The padding value for the bottom side.
    /// * `left` - The padding value for the left side.
    /// * `right` - The padding value for the right side.
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = Client::default();
    /// client.set_paddings(42, 0, 0, 0);
    /// assert_eq!(client.padding_top, 42);
    /// assert_eq!(client.padding_bottom, 0);
    /// assert_eq!(client.padding_left, 0);
    /// assert_eq!(client.padding_right, 0);
    /// ```
    pub fn set_paddings(&mut self, top: u32, bottom: u32, left: u32, right: u32) {
        self.padding_top = top;
        self.padding_bottom = bottom;
        self.padding_left = left;
        self.padding_right = right;
    }

    /// Returns whether the client needs control.
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = Client::default();
    /// assert_eq!(client.needs_control(), false);
    /// client.set_type(ClientType::Dock);
    /// assert_eq!(client.needs_control(), false);
    /// ```
    #[inline]
    pub fn is_controlled(&self) -> bool {
        self.is_controlled
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    #[inline]
    pub fn has_state(&self, state: &ClientState) -> bool {
        self.states.contains(state)
    }

    /// Sets the specified state for the client.
    ///
    /// # Arguments
    ///
    /// * `state` - The state to set.
    /// * `action` - The action to perform (add, remove, toggle).
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the state is added, `Ok(false)` if the state is removed,
    /// and `Err(String)` if the action is unknown or invalid.
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = Client::default();
    /// let state = State::Maximized;
    /// assert_eq!(client.set_state(state, ClientState::Add), Ok(true));
    /// assert_eq!(client.set_state(state, ClientState::Remove), Ok(false));
    /// ```
    pub fn set_state(&mut self, state: ClientState, action: Operation) -> Result<bool, String> {
        match action {
            Operation::Add => {
                if !self.has_state(&state) {
                    self.states.push(state);
                }
                Ok(true)
            },
            Operation::Remove => {
                if let Some(i) = self.states.iter().position(|s| *s == state) {
                    self.states.swap_remove(i);
                }
                Ok(false)
            },
            Operation::Toggle => {
                let has = self.has_state(&state);
                if let Some(i) = self.states.iter().position(|s| *s == state) {
                    self.states.remove(i);
                } else {
                    self.states.push(state);
                }
                Ok(!has)
            },
            Operation::Unknown => Err("Invalid state.".to_string()),
        }
    }

    /// Gets the type of the client.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `ClientType`.
    ///
    /// # Example
    ///
    /// ```
    /// let client = Client::default();
    /// assert_eq!(*client.get_type(), ClientType::Normal);
    /// ```
    pub fn get_type(&self) -> &ClientType {
        &self.r#type
    }

    /// Sets the client type and performs additional configurations if needed.
    ///
    /// # Arguments
    ///
    /// * `r#type` - The new client type to set.
    ///
    /// # Example
    ///
    /// ```
    /// let mut client = Client::default();
    /// client.set_type(ClientType::Dock);
    /// assert_eq!(client.r#type, ClientType::Dock);
    /// ```
    pub fn set_type(&mut self, r#type: ClientType) {
        self.r#type = r#type;
        // Some types need that the client has additional configurations.
        match self.r#type {
            ClientType::Dock => {
                self.is_controlled = false;
            },
            _ => {},
        }
    }

    pub fn set_border_color(&self, conn: &ewmh::Connection, color: u32) {
        xcb::change_window_attributes(
            conn,
            self.wid,
            &[(xcb::CW_BORDER_PIXEL, color)],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_visible() {
        let mut client = Client::default();
        client.is_visible = true;
        assert_eq!(client.is_visible(), true);

        let mut client = Client::default();
        client.is_visible = false;
        assert_eq!(client.is_visible(), false);
    }

    #[test]
    fn test_has_state() {
        let mut client = Client::default();
        client.states = vec![ClientState::Fullscreen];
        assert_eq!(client.has_state(&ClientState::Fullscreen), true);
        assert_eq!(client.has_state(&ClientState::Maximized), false);
    }

    #[test]
    fn test_set_state_add() {
        let mut client = Client::default();
        client.states = vec![ClientState::Fullscreen];
        let result = client.set_state(ClientState::Maximized, Operation::Add);
        assert_eq!(result, Ok(true));
        assert_eq!(client.states, vec![ClientState::Fullscreen, ClientState::Maximized]);
    }

    #[test]
    fn test_set_state_remove() {
        let mut client = Client::default();
        client.states = vec![ClientState::Fullscreen, ClientState::Maximized];
        let result = client.set_state(ClientState::Fullscreen, Operation::Remove);
        assert_eq!(result, Ok(false));
        assert_eq!(client.states, vec![ClientState::Maximized]);
    }

    #[test]
    fn test_set_state_toggle() {
        let mut client = Client::default();
        client.states = vec![ClientState::Fullscreen];
        let result = client.set_state(ClientState::Fullscreen, Operation::Toggle);
        assert_eq!(result, Ok(false));
        assert_eq!(client.states, vec![]);

        let mut client = Client::default();
        client.states = vec![];
        let result = client.set_state(ClientState::Fullscreen, Operation::Toggle);
        assert_eq!(result, Ok(true));
        assert_eq!(client.states, vec![ClientState::Fullscreen]);
    }

    #[test]
    fn test_set_state_invalid() {
        let mut client = Client::default();
        let result = client.set_state(ClientState::Fullscreen, Operation::Unknown);
        assert_eq!(result, Err("Invalid state.".to_string()));
    }

    #[test]
    fn test_set_paddings() {
        let mut client = Client::default();
        client.set_paddings(42, 0, 0, 0);
        assert_eq!(client.padding_top, 42);
        assert_eq!(client.padding_bottom, 0);
        assert_eq!(client.padding_left, 0);
        assert_eq!(client.padding_right, 0);
    }

    #[test]
    fn test_set_type_dock() {
        let mut client = Client::default();
        client.set_type(ClientType::Dock);
        assert_eq!(client.r#type, ClientType::Dock);
        assert_eq!(client.is_controlled, false);
    }

    #[test]
    fn test_get_type() {
        let mut client = Client::default();
        client.r#type = ClientType::Normal;
        assert_eq!(*client.get_type(), ClientType::Normal);
    }

    #[test]
    fn test_needs_controll() {
        let mut client = Client::default();
        client.is_controlled = false;
        assert_eq!(client.is_controlled(), false);
    }
}
