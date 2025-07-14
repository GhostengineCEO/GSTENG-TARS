use gsteng::config::config::Config;

#[test]
fn load_default_config() {
    let path = std::path::PathBuf::from("test_config.toml");
    let cfg = Config::load(&path).expect("load default");
    assert!(!cfg.hardware.port.is_empty());
    std::fs::remove_file(path).unwrap();
}
