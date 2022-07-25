use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Arg<'a> {
    Flag(Flag<'a>),
    Positional(Positional<'a>),
    Optional(Optional<'a>),
}

impl<'a> Arg<'a> {
    pub fn as_flag_ref(&self) -> &Flag {
        match self {
            Arg::Flag(f) => f,
            Arg::Optional(o) => o.get_flag_ref(),
            Arg::Positional(_) => panic!("positional cannot be accessed as flag")
        }
    }
}

impl<'a> Display for Arg<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Arg::Flag(a) => write!(f, "{}", a),
            Arg::Positional(a) => write!(f, "{}", a),
            Arg::Optional(a) => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Positional<'a> {
    name: &'a str,
}

impl<'a> Positional<'a> {
    pub fn new(s: &'a str) -> Self {
        Positional { name: s, }
    }
}

impl<'a> Display for Positional<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "<{}>", self.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct Flag<'a> {
    name: &'a str,
    switch: Option<char>,
}

impl<'a> Flag<'a> {
    pub fn new(s: &'a str) -> Self {
        Flag { name: s, switch: None, }
    }

    pub fn switch(mut self, c: char) -> Self {
        self.switch = Some(c);
        self
    }

    pub fn get_name_ref(&self) -> &str {
        self.name
    }

    pub fn get_switch_ref(&self) -> Option<&char> {
        self.switch.as_ref()
    }
}

impl<'a> Display for Flag<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "--{}", self.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct Optional<'a> {
    option: Flag<'a>,
    value: Positional<'a>,
}

impl<'a> Optional<'a> {
    pub fn new(s: &'a str) -> Self {
        Optional { option: Flag::new(s), value: Positional::new(s), }
    }

    pub fn value(mut self, s: &'a str) -> Self {
        self.value.name = s;
        self
    }

    pub fn switch(mut self, c: char) -> Self {
        self.option.switch = Some(c);
        self
    }

    pub fn get_flag_ref(&self) -> &Flag {
        &self.option
    }

    pub fn _get_pos_ref(&self) -> &Positional {
        &self.value
    }
}

impl<'a> Display for Optional<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "{} {}", self.option, self.value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn positional_new() {
        let ip = Positional::new("ip");
        assert_eq!(ip, Positional {
            name: "ip",
        });

        let version = Positional::new("version");
        assert_eq!(version, Positional {
            name: "version",
        });
    }

    #[test]
    fn positional_disp() {
        let ip = Positional::new("ip");
        assert_eq!(ip.to_string(), "<ip>");

        let topic = Positional::new("topic");
        assert_eq!(topic.to_string(), "<topic>");
    }

    #[test]
    fn flag_new() {
        let help = Flag::new("help").switch('h');
        assert_eq!(help, Flag {
            name: "help",
            switch: Some('h'),
        });
        assert_eq!(help.get_switch_ref(), Some(&'h'));
        assert_eq!(help.get_name_ref(), "help");

        let version = Flag::new("version");
        assert_eq!(version, Flag {
            name: "version",
            switch: None,
        });
        assert_eq!(version.get_switch_ref(), None);
        assert_eq!(version.get_name_ref(), "version");
    }

    #[test]
    fn flag_disp() {
        let help = Flag::new("help");
        assert_eq!(help.to_string(), "--help");

        let version = Flag::new("version");
        assert_eq!(version.to_string(), "--version");
    }

    #[test]
    fn optional_new() {
        let code = Optional::new("code");
        assert_eq!(code, Optional {
            option: Flag::new("code"),
            value: Positional::new("code"),
        });
        assert_eq!(code.get_flag_ref().get_switch_ref(), None);

        let version = Optional::new("color").value("rgb");
        assert_eq!(version, Optional {
            option: Flag::new("color"),
            value: Positional::new("rgb"),
        });
        assert_eq!(version.get_flag_ref().get_switch_ref(), None);

        let version = Optional::new("color").value("rgb").switch('c');
        assert_eq!(version, Optional {
            option: Flag::new("color").switch('c'),
            value: Positional::new("rgb"),
        });
        assert_eq!(version.get_flag_ref().get_switch_ref(), Some(&'c'));

        assert_eq!(version._get_pos_ref(), &Positional::new("rgb"));
    }

    #[test]
    fn optional_disp() {
        let code = Optional::new("code");
        assert_eq!(code.to_string(), "--code <code>");

        let color = Optional::new("color").value("rgb");
        assert_eq!(color.to_string(), "--color <rgb>");

        let color = Optional::new("color").value("rgb").switch('c');
        assert_eq!(color.to_string(), "--color <rgb>");
    }

    #[test]
    fn arg_disp() {
        let command = Arg::Positional(Positional::new("command"));
        assert_eq!(command.to_string(), "<command>");

        let help = Arg::Flag(Flag::new("help"));
        assert_eq!(help.to_string(), "--help");

        assert_eq!(help.as_flag_ref().to_string(), "--help");

        let color = Arg::Optional(Optional::new("color").value("rgb"));
        assert_eq!(color.to_string(), "--color <rgb>");

        assert_eq!(color.as_flag_ref().get_name_ref(), "color");
    }

    #[test]
    #[should_panic]
    fn arg_impossible_pos_as_flag() {
        let command = Arg::Positional(Positional::new("command"));
        let _ = command.as_flag_ref();
    }
}