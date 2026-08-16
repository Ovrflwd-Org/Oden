fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Oden");
        res.set("FileDescription", "Oden");
        if let Err(err) = res.compile() {
            println!("cargo:warning=failed to embed Windows exe icon: {err}");
        }
    }
}
