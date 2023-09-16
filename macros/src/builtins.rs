#[allow(dead_code)] // FIXME
pub(crate) struct Builtin {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
}

#[allow(dead_code)] // FIXME
pub(crate) const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "c",
        description: "Require an exact match with a character.",
    },
    Builtin {
        name: "s",
        description: "Require an exact match with a sequence of characters.",
    },
    Builtin {
        name: "FuckingShit",
        description: "Eat my assholes.",
    },
];
