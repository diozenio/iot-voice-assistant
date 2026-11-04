use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Runtime,
};

pub fn setup<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let open_dashboard =
        MenuItem::with_id(app, "open_dashboard", "Abrir Dashboard", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Fechar", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_dashboard, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open_dashboard" => crate::window::focus_main(app),
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
