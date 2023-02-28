use darling::FromMeta;

#[derive(Debug, Clone, Copy, FromMeta)]
pub enum ConfigFileLoader {
    Toml,
    Json,
    Yaml,
}

pub struct LoaderPaths {
    pub from_str: syn::Path,
    pub to_string: syn::Path,
    pub to_string_pretty: syn::Path,

    pub de_error_path: syn::Path,
}

#[derive(Debug, FromMeta)]
pub struct ItemArgs {
    pub loader: Option<ConfigFileLoader>,
    pub debug: Option<bool>,
}

impl From<ConfigFileLoader> for LoaderPaths {
    fn from(value: ConfigFileLoader) -> Self {
        fn simple(package: &str) -> LoaderPaths {
            LoaderPaths {
                from_str: syn::parse_str(&format!(
                    "{package}::from_str"
                ))
                .unwrap(),
                de_error_path: syn::parse_str(&format!(
                    "{package}::de::Error"
                ))
                .unwrap(),
                to_string: syn::parse_str(&format!(
                    "{package}::to_string"
                ))
                .unwrap(),
                to_string_pretty: syn::parse_str(&format!(
                    "{package}::to_string_pretty"
                ))
                .unwrap(),
            }
        }

        match value {
            ConfigFileLoader::Json => simple("serde_json"),
            ConfigFileLoader::Toml => simple("toml"),
            ConfigFileLoader::Yaml => simple("serde_yaml"),
        }
    }
}
