pub fn window_clients() -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .title("Clients")
        .default_width(400)
        .default_height(300)
        .build();

    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    window.set_child(Some(&list_box));

    window
}
