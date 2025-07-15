/// Commands to be sent over the server
pub enum Commands {
    /// Add data to the current stats
    INPUT,
    OUTPUT,
    EXIT,
    ERROR
}

impl Commands {
    pub fn to_string<'a>(self) -> &'a str {
        match self {
            Self::INPUT => "INPUT",
            Self::OUTPUT => "OUTPUT",
            Self::EXIT => "EXIT",
            Self::ERROR => "ERROR"
        }
    }
}

pub trait CommandTraits {
    fn to_command(&self) -> Commands;
}

impl CommandTraits for String {
    fn to_command(&self) -> Commands {
        match self.as_str() {
            "INPUT" => Commands::INPUT,
            "OUTPUT" => Commands::OUTPUT,
            "EXIT" => Commands::EXIT,
            _ => Commands::ERROR
        }
    }
}

impl CommandTraits for str {
    fn to_command(&self) -> Commands {
        match self.replace("!", "").as_str() {
            "INPUT" => Commands::INPUT,
            "OUTPUT" => Commands::OUTPUT,
            "EXIT" => Commands::EXIT,
            _ => Commands::ERROR
        }
    }
}
