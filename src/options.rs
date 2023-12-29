use std::{sync::{Arc, Mutex}, thread::sleep, time::Duration};

use crate::models::OperatorConfig;
use anyhow::{anyhow, Result};
use cursive::{
    view::{Nameable, Resizable},
    views::{Button, Dialog, DummyView, EditView, LinearLayout, NamedView, SelectView, TextView},
    Cursive,
};
use cursive_aligned_view::Alignable;
use rusqlite::Connection;

fn add_option(connection: Arc<Mutex<Connection>>) -> Result<()> {
    if let Ok(conn) = connection.lock() {
        conn.execute(
            "INSERT INTO operatorconfig (name, call, grid, cqz, ituz, dxcc, cont) VALUES ('New Option', '', '', '', '', '', '')",
            (),
        )?;
        Ok(())
    } else {
        Err(anyhow!("Could not lock connection"))
    }
}

fn update_select(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    let connection = connection.clone();
    let options = if let Ok(conn) = connection.lock() {
        let mut stmt = conn.prepare("SELECT * FROM operatorconfig ORDER BY id ASC")?;
        let mut options: Vec<OperatorConfig> = Vec::new();
        let options_out = stmt.query_map((), |row| {
            Ok(OperatorConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                call: row.get(2)?,
                grid: row.get(3)?,
                cqz: row.get(4)?,
                ituz: row.get(5)?,
                dxcc: row.get(6)?,
                cont: row.get(7)?,
            })
        })?;
        for option in options_out {
            options.push(option?);
        }
        options
    } else {
        return Err(anyhow!("Could not lock connection"));
    };
    s.call_on_name("options", move |view: &mut SelectView<OperatorConfig>| {
        for _ in 0..view.len() {
            view.remove_item(0);
        }
        view.add_all(options.iter().map(|opt| (opt.name.clone(), opt.clone())));
    })
    .ok_or(anyhow!("Failed to update options"))?;
    Ok(())
}

fn save(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    let id = s
        .call_on_name("id", |view: &mut EditView| view.get_content())
        .unwrap();
    let name = s
        .call_on_name("name", |view: &mut EditView| view.get_content())
        .unwrap();
    let call = s
        .call_on_name("callsign", |view: &mut EditView| view.get_content())
        .unwrap();
    let grid = s
        .call_on_name("grid", |view: &mut EditView| view.get_content())
        .unwrap();
    let cqz = s
        .call_on_name("cqz", |view: &mut EditView| view.get_content())
        .unwrap();
    let ituz = s
        .call_on_name("ituz", |view: &mut EditView| view.get_content())
        .unwrap();
    let dxcc = s
        .call_on_name("dxcc", |view: &mut EditView| view.get_content())
        .unwrap();
    let cont = s
        .call_on_name("cont", |view: &mut EditView| view.get_content())
        .unwrap();
    if let Ok(conn) = connection.lock() {
        let stmt = "UPDATE operatorconfig SET name = ?, call = ?, grid = ?, cqz = ?, ituz = ?, dxcc = ?, cont = ? WHERE id = ?";
        conn.execute(
            stmt,
            (
                name,
                call,
                grid,
                cqz,
                ituz,
                dxcc,
                cont,
                id,
            ),
        )?;
        let cb_sink = s.cb_sink().clone();
        std::thread::spawn(move || {
            cb_sink.send(Box::new(|s: &mut Cursive| {
                s.add_layer(Dialog::text("Save Complete!").title("Saved"));
            })).unwrap();
            sleep(Duration::from_secs(2));
            cb_sink.send(Box::new(|s: &mut Cursive| {
                s.pop_layer();
            })).unwrap();
        });
        Ok(())
    } else {
        Err(anyhow!("Could not lock connection"))
    }
}

fn delete_option(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    let id = s
        .call_on_name("id", |view: &mut EditView| view.get_content())
        .unwrap();
    if let Ok(conn) = connection.lock() {
        let stmt = "DELETE FROM operatorconfig WHERE id = ?";
        conn.execute(
            stmt,
            (
                id,
            ),
        )?;
        let cb_sink = s.cb_sink().clone();
        std::thread::spawn(move || {
            cb_sink.send(Box::new(|s: &mut Cursive| {
                s.add_layer(Dialog::text("Delete Complete!").title("Deleted"));
            })).unwrap();
            sleep(Duration::from_secs(2));
            cb_sink.send(Box::new(|s: &mut Cursive| {
                s.pop_layer();
            })).unwrap();
        });
        Ok(())
    } else {
        Err(anyhow!("Could not lock connection"))
    }
}


fn delete(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    s.add_layer(Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new("Are you sure you want to delete this option?"))
            .child(
                LinearLayout::horizontal()
                    .child(Button::new("Yes", move |s| {
                        delete_option(s, connection.clone()).unwrap();
                        update_select(s, connection.clone()).unwrap();
                        s.call_on_name("options", move |view: &mut SelectView<OperatorConfig>| {
                            view.set_selection(0)
                        }).unwrap()(s);
                        s.pop_layer();
                    }))
                    .child(Button::new("No", |s| {
                        s.pop_layer();
                    })),
            ),
    ));
    Ok(())
}

pub fn options(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    let select_view: NamedView<SelectView<OperatorConfig>> = SelectView::new().on_select(|s: &mut Cursive, item: &OperatorConfig| {
        s.call_on_name("id", move |view: &mut EditView| {
            view.set_content(item.id.to_string());
        });
        s.call_on_name("name", move |view: &mut EditView| {
            view.set_content(item.name.to_string());
        });
        s.call_on_name("callsign", move |view: &mut EditView| {
            view.set_content(item.call.to_string());
        });
        s.call_on_name("grid", move |view: &mut EditView| {
            view.set_content(item.grid.to_string());
        });
        s.call_on_name("cqz", move |view: &mut EditView| {
            view.set_content(item.cqz.to_string());
        });
        s.call_on_name("ituz", move |view: &mut EditView| {
            view.set_content(item.ituz.to_string());
        });
        s.call_on_name("dxcc", move |view: &mut EditView| {
            view.set_content(item.dxcc.to_string());
        });
        s.call_on_name("cont", move |view: &mut EditView| {
            view.set_content(item.cont.to_string());
        });
    }).with_name("options");
    let add_connection = connection.clone();
    let save_connection = connection.clone();
    let delete_connection = connection.clone();
    s.pop_layer();
    s.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(
                    LinearLayout::vertical()
                        .child(select_view)
                        .child(DummyView)
                        .child(Button::new("Add", move |s| {
                            add_option(add_connection.clone()).unwrap();
                            update_select(s, add_connection.clone()).unwrap();
                            s.call_on_name("options", move |view: &mut SelectView<OperatorConfig>| {
                                view.set_selection(view.len()-1)
                            }).unwrap()(s);
                        })),
                )
                .child(DummyView)
                .child(
                    LinearLayout::vertical()
                        .child(
                            LinearLayout::horizontal()
                                .child(
                                    Dialog::around(EditView::new().disabled().with_name("id").fixed_width(20))
                                        .title("ID"),
                                )
                                .child(
                                    Dialog::around(
                                        EditView::new().with_name("name").fixed_width(20),
                                    )
                                    .title("Name"),
                                )
                                .align_center(),
                        )
                        .child(
                            LinearLayout::horizontal()
                                .child(
                                    Dialog::around(EditView::new().with_name("callsign").fixed_width(20))
                                        .title("Callsign"),
                                )
                                .child(
                                    Dialog::around(
                                        EditView::new().with_name("grid").fixed_width(20),
                                    )
                                    .title("Grid Square"),
                                )
                                .align_center(),
                        )
                        .child(
                            LinearLayout::horizontal()
                                .child(
                                    Dialog::around(EditView::new().with_name("cqz").fixed_width(10))
                                        .title("CQZ"),
                                )
                                .child(
                                    Dialog::around(
                                        EditView::new().with_name("ituz").fixed_width(10),
                                    )
                                    .title("ITUZ"),
                                )
                                .child(
                                    Dialog::around(EditView::new().with_name("dxcc").fixed_width(10))
                                        .title("DXCC"),
                                )
                                .child(
                                    Dialog::around(
                                        EditView::new().with_name("cont").fixed_width(10),
                                    )
                                    .title("Cont"),
                                )
                                .align_center(),
                        )
                        .child(
                            LinearLayout::horizontal()
                                .child(Button::new("Save", move |s| {
                                    save(s, save_connection.clone()).unwrap();
                                    update_select(s, save_connection.clone()).unwrap();
                                }))
                                .child(DummyView)
                                .child(Button::new("Delete", move |s| {
                                    delete(s, delete_connection.clone()).unwrap();
                                }))
                                .align_center(),
                        ),
                )
                .align_center(),
        )
        .title("Options"),
    );
    update_select(s, connection.clone())?;
    s.call_on_name("options", move |view: &mut SelectView<OperatorConfig>| {
        view.set_selection(0)
    }).unwrap()(s);
    Ok(())
}
