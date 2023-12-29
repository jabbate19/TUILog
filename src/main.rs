use anyhow::{anyhow, Result};
use cursive::event::Key;
use cursive::menu::Tree;
use cursive::views::Dialog;
use homedir::get_my_home;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::fs;
mod log;
use log::new_log;

mod logbook;
use logbook::make_table;

mod models;

mod options;
use options::options;

fn main() -> Result<()> {
    let mut homepath = get_my_home().unwrap().unwrap();
    homepath.push(".tuilog");
    if let Ok(folder_data) = fs::metadata(&homepath) {
        if !folder_data.is_dir() {
            return Err(anyhow!("ERR: ~/.tuilog is not a folder. Please delete/rename the item."));
        }
    } else {
        fs::create_dir(&homepath)?;
    }
    homepath.push("tuilog.db");
    let connection = Connection::open(homepath)?;
    let query = "
        CREATE TABLE IF NOT EXISTS operatorconfig (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, call TEXT, grid TEXT, cqz TEXT, ituz TEXT, dxcc TEXT, cont TEXT);
    ";
    connection.execute(query, ())?;
    let query = "
        CREATE TABLE IF NOT EXISTS logs (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp TEXT, call TEXT, rsttx TEXT, rstrx TEXT, band TEXT, frequency TEXT, mode TEXT, power TEXT, comments TEXT, operator_config INTEGER NOT NULL REFERENCES operatorConfig(id));
    ";
    connection.execute(query, ())?;

    let connection = Arc::new(Mutex::new(connection));

    let mut siv = cursive::default();
    siv.set_autorefresh(true);

    let new_log_conn = connection.clone();
    let logbook_conn = connection.clone();
    let options_conn = connection.clone();

    siv.menubar().add_subtree(
        "File",
        Tree::new()
            .leaf("New Log", move |s| {
                new_log(s, new_log_conn.clone()).unwrap()
            })
            .leaf("Logbook", move |s| {
                make_table(s, logbook_conn.clone()).unwrap()
            })
            .leaf("Options", move |s| {
                options(s, options_conn.clone()).unwrap()
            })
            .leaf("Quit", |s| s.quit()),
    );

    siv.add_global_callback(Key::Esc, |s| s.select_menubar());

    siv.add_layer(Dialog::text("TUILog v1.0.0").title("TUILog"));

    siv.run();

    Ok(())
}
