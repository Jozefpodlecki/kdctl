fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("ProductName", "KDCTL VM Host Service");
        res.set("FileDescription", "Kernel driver controller host VM service");
        res.set("CompanyName", "Jozef Podlecki");
        res.set("LegalCopyright", "Copyright © 2026 Jozef Podlecki");
        
        res.compile().unwrap();
    }
}