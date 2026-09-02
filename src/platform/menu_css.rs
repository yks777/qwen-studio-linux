/// GTK CSS for the HeaderBar menubar (injected in `platform::menu::setup`).
/// Ensures headerbar has visible height even on Wayland compact themes.
pub const MENU_CSS: &str = r#"
headerbar {
    padding: 0 8px;
    min-height: 36px;
}
headerbar button {
    min-height: 28px;
    min-width: 28px;
    padding: 4px 8px;
}
"#;
