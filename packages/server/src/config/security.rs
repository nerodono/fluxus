entity! {
    struct SecurityConfig {
        // Used as the alternative authentication method (without involving database)
        universal_password: Option<String>,
    }
}
