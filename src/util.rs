use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};

pub struct LocalizedString {
    pub en: &'static str,
}

pub trait CreateApplicationCommandExt {
    fn localized_name(&mut self, str: LocalizedString) -> &mut Self;
    fn localized_desc(&mut self, str: LocalizedString) -> &mut Self;
}

impl CreateApplicationCommandExt for CreateApplicationCommand {
    fn localized_name(&mut self, str: LocalizedString) -> &mut Self {
        self.name(str.en).name_localized("en-US", str.en)
    }

    fn localized_desc(&mut self, str: LocalizedString) -> &mut Self {
        self.description(str.en)
            .description_localized("en-US", str.en)
    }
}

pub trait CreateApplicationCommandOptionExt {
    fn localized_name(&mut self, str: LocalizedString) -> &mut Self;
    fn localized_desc(&mut self, str: LocalizedString) -> &mut Self;
}

impl CreateApplicationCommandOptionExt for CreateApplicationCommandOption {
    fn localized_name(&mut self, str: LocalizedString) -> &mut Self {
        self.name(str.en).name_localized("en-US", str.en)
    }

    fn localized_desc(&mut self, str: LocalizedString) -> &mut Self {
        self.description(str.en)
            .description_localized("en-US", str.en)
    }
}

impl LocalizedString {
    pub fn any_eq(&self, str: impl AsRef<str>) -> bool {
        self.en == str.as_ref()
    }
}
