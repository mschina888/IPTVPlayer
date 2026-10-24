fn main() {
    let cfg = slint_build::CompilerConfiguration::new().with_style("fluent".into());
    slint_build::compile_with_config("ui/app.slint", cfg).unwrap();

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/app.ico");
    res.set("ProductName", "IPTV Player for Windows 11 x64");
    res.set("FileDescription", "IPTV Player for Windows 11 x64");
    res.set("CompanyName", "MSChina888");
    res.set("LegalCopyright", "MSChina888");
    res.compile().unwrap();
}
