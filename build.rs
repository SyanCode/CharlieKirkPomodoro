fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    // Icône de l'exécutable Windows (explorateur, barre des tâches, raccourcis).
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "CharlieKirkPomodoro");
        res.set("FileDescription", "CharlieKirkPomodoro");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icône non intégrée : {e}");
        }
    }
}
