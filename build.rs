fn main() {
    // 把 assets/app.ico 作为 exe 图标写进资源，资源管理器里才不是白纸。
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/app.ico");
        resource.set("FileDescription", "键盘映射");
        resource.set("ProductName", "键盘映射");
        resource.set("LegalCopyright", "MIT");
        if let Err(error) = resource.compile() {
            println!("cargo:warning=图标资源编译失败: {error}");
        }
    }
    println!("cargo:rerun-if-changed=assets/app.ico");
}
