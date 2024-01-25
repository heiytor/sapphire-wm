use crate::keyboard::FnOnKeypress;

pub struct Keybinding {
    /// Callback function to be executed when the key is pressed.
    pub callback: Box<dyn FnOnKeypress>,

    /// Represents the keyboard state when the key was pressed. The state typically reflects
    /// which modifier keys the user pressed simultaneously.
    ///
    /// For instance, pressing "Shift" should set the state as "4", pressing with "Shift" + "Ctrl"
    /// should set the state as "4 | 1" or "5".
    pub modkeys: u16,

    /// Specifies the key that triggers the keybind. This is a string representation of a
    /// keysymbol, which later corresponds to a keycode, an unsigned 8-bit integer representing
    /// the physical layout of a keyboard. You can use tools like `xev` to obtain more information
    /// about allowed key strings and keycodes.
    pub key: String,

    /// Description of the keybind.
    pub description: String,

    /// Group to which the keybind belongs.
    pub group: String,
}

impl Keybinding {
    /// Returns a `KeybindingBuilder` used to construct a `Keybinding`. All default values are set to 0 or
    /// an empty string.
    ///
    /// Use `KeybindingBuilder::execute()` to set the callback and complete the build process.
    pub fn new() -> KeybindingBuilder {
        KeybindingBuilder::new()
    }
}

pub struct KeybindingBuilder {
    modkeys: u16,
    key: String,
    description: String,
    group: String,
}

#[allow(dead_code)]
impl KeybindingBuilder {
    /// Creates a new `KeybindingBuilder` with default values.
    fn new() -> Self {
        KeybindingBuilder {
            modkeys: 0,
            key: String::new(),
            description: String::new(),
            group: String::new(),
        }
    }

    /// Sets the modkeys and key to trigger the keybind.
    pub fn on(&mut self, modkeys: &[u16], keys: &str) -> &mut Self {
        self.modkeys = modkeys.iter().fold(0, |acc, &m| acc | m);
        self.key = keys.to_owned();
        self
    }

    /// Sets the group of the keybind.
    pub fn group(&mut self, group: &str) -> &mut Self {
        self.group = group.to_owned();
        self
    }

    /// Sets the description of the keybind.
    pub fn description(&mut self, desc: &str) -> &mut Self {
        self.description = desc.to_owned();
        self
    }

    /// Sets the callback and finalizes the build process.
    pub fn execute(&mut self, callback: Box<dyn FnOnKeypress>) -> Keybinding {
        Keybinding {
            modkeys: self.modkeys,
            key: self.key.to_owned(),
            callback,
            description: self.description.to_owned(),
            group: self.group.to_owned(),
        }
    }
}
