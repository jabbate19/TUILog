use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use chrono::Utc;
use cursive::{
    align::HAlign,
    view::{Nameable, Resizable},
    views::{Button, Dialog, DummyView, EditView, LinearLayout, SelectView},
    Cursive,
};
use cursive_aligned_view::Alignable;
use rusqlite::Connection;

use crate::models::OperatorConfig;

fn add_log(s: &mut Cursive, connection: Arc<Mutex<Connection>>) {
    let callsign = s
        .call_on_name("callsign", |view: &mut EditView| view.get_content())
        .unwrap();
    let profile = s
        .call_on_name("profile", |view: &mut Button| {
            let val = view.label();
            let parens = val.find('(').unwrap();
            val[1..parens].to_string()
        })
        .unwrap();
    let band = s
        .call_on_name("band", |view: &mut Button| {
            view.label()
                .trim_matches(|c| c == '<' || c == '>')
                .to_string()
        })
        .unwrap();
    let band_str = band.clone();
    let band_str = band_str.as_str();
    let frequency = s
        .call_on_name("frequency", |view: &mut EditView| view.get_content())
        .unwrap();
    let mode = s
        .call_on_name("mode", |view: &mut Button| {
            view.label()
                .trim_matches(|c| c == '<' || c == '>')
                .to_string()
        })
        .unwrap();
    let rsttx = s
        .call_on_name("rsttx", |view: &mut EditView| view.get_content())
        .unwrap();
    let rstrx = s
        .call_on_name("rstrx", |view: &mut EditView| view.get_content())
        .unwrap();
    let power = s
        .call_on_name("power", |view: &mut EditView| view.get_content())
        .unwrap();
    let comments = s
        .call_on_name("comments", |view: &mut EditView| view.get_content())
        .unwrap();
    if let Ok(conn) = connection.lock() {
        let stmt = "INSERT INTO logs (timestamp, call, rsttx, rstrx, band, frequency, mode, power, comments, operator_config) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
        let timestamp = Utc::now()
            .naive_utc()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        conn.execute(
            stmt,
            (
                timestamp,
                callsign,
                rsttx,
                rstrx,
                band,
                frequency,
                mode,
                power,
                comments,
                profile,
            ),
        )
        .unwrap();
    }
    s.call_on_name("callsign", |view: &mut EditView| {
        view.set_content("");
    });
    s.call_on_name("comments", |view: &mut EditView| {
        view.set_content("");
    });

    s.call_on_name("rsttx", |view: &mut EditView| {
        view.set_content(match band_str {
            "SSB" => "59",
            "CW" => "599",
            "FT8" => "599",
            _ => "59",
        });
        view.set_max_content_width(Some(match band_str {
            "SSB" => 2,
            "CW" => 3,
            "FT8" => 3,
            _ => 2,
        }));
    });
    s.call_on_name("rstrx", |view: &mut EditView| {
        view.set_content(match band_str {
            "SSB" => "59",
            "CW" => "599",
            "FT8" => "599",
            _ => "59",
        });
        view.set_max_content_width(Some(match band_str {
            "SSB" => 2,
            "CW" => 3,
            "FT8" => 3,
            _ => 2,
        }));
    });
}

fn select_band(s: &mut Cursive) {
    let mut select = SelectView::new().h_align(HAlign::Center);
    select.add_item("160M", "160M");
    select.add_item("80M", "80M");
    select.add_item("60M", "60M");
    select.add_item("40M", "40M");
    select.add_item("30M", "30M");
    select.add_item("20M", "20M");
    select.add_item("17M", "17M");
    select.add_item("15M", "15M");
    select.add_item("12M", "12M");
    select.add_item("10M", "10M");
    select.add_item("6M", "6M");
    select.add_item("2M", "2M");
    select.add_item("70CM", "70CM");
    select.set_on_submit(|s, band: &str| {
        s.pop_layer();
        s.call_on_name("band", |view: &mut Button| {
            view.set_label(band);
        });
        s.call_on_name("frequency", |view: &mut EditView| {
            view.set_content(match band {
                "160M" => "1.8",
                "80M" => "3.5",
                "60M" => "5.3",
                "40M" => "7.0",
                "30M" => "10.1",
                "20M" => "14.0",
                "17M" => "18.1",
                "15M" => "21.0",
                "12M" => "24.9",
                "10M" => "28.0",
                "6M" => "50.0",
                "2M" => "144.0",
                "70CM" => "432.0",
                _ => "14.0",
            });
        });
    });
    s.add_layer(Dialog::around(select).title("Select Band"));
}

fn select_profile(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    let mut select = SelectView::new().h_align(HAlign::Center);
    if let Ok(conn) = connection.lock() {
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
            let option = option?;
            let label = format!("{} ({})", option.id, option.name);
            select.add_item(label.clone(), label);
        }
    } else {
        return Err(anyhow!("Could not lock connection"));
    };
    select.set_on_submit(|s, profile: &str| {
        s.pop_layer();
        s.call_on_name("profile", |view: &mut Button| {
            view.set_label(profile);
        });
    });
    s.add_layer(Dialog::around(select).title("Select Profile"));
    Ok(())
}

fn select_mode(s: &mut Cursive) {
    let mut select = SelectView::new().h_align(HAlign::Center);
    select.add_item("SSB", "SSB");
    select.add_item("USB", "USB");
    select.add_item("LSB", "LSB");
    select.add_item("CW", "CW");
    select.add_item("FT8", "FT8");
    select.set_on_submit(|s, band: &str| {
        s.pop_layer();
        s.call_on_name("mode", |view: &mut Button| {
            view.set_label(band);
        });
        s.call_on_name("rsttx", |view: &mut EditView| {
            view.set_content(match band {
                "SSB" => "59",
                "CW" => "599",
                "FT8" => "599",
                _ => "59",
            });
            view.set_max_content_width(Some(match band {
                "SSB" => 2,
                "CW" => 3,
                "FT8" => 3,
                _ => 2,
            }));
        });
        s.call_on_name("rstrx", |view: &mut EditView| {
            view.set_content(match band {
                "SSB" => "59",
                "CW" => "599",
                "FT8" => "599",
                _ => "59",
            });
            view.set_max_content_width(Some(match band {
                "SSB" => 2,
                "CW" => 3,
                "FT8" => 3,
                _ => 2,
            }));
        });
    });
    s.add_layer(Dialog::around(select).title("Select Band"));
}

pub fn new_log(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    let profile_connection = connection.clone();
    s.pop_layer();
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(
                    LinearLayout::horizontal().child(
                    Dialog::around(
                        EditView::new()
                            .with_name("callsign")
                            .fixed_width(10)
                            .align_center(),
                    )
                    .title("Callsign"))
                    .child(
                        Dialog::around(Button::new("", move |s| select_profile(s, profile_connection.clone()).unwrap()).with_name("profile"))
                                .title("Profile")
                    ).align_center()
                )
                .child(
                    LinearLayout::horizontal()
                        .child(
                            Dialog::around(Button::new("", select_band).with_name("band"))
                                .title("Band"),
                        )
                        .child(
                            Dialog::around(EditView::new().with_name("frequency").fixed_width(10).align_center())
                                .title("Frequency"),
                        )
                        .child(
                            Dialog::around(Button::new("", select_mode).with_name("mode"))
                                .title("Mode"),
                        ).align_center(),
                )
                .child(
                    LinearLayout::horizontal()
                        .child(
                            Dialog::around(EditView::new().with_name("rsttx").fixed_width(5).align_center())
                                .title("RST TX"),
                        )
                        .child(
                            Dialog::around(EditView::new().with_name("rstrx").fixed_width(5).align_center())
                                .title("RST RX"),
                        )
                        .child(
                            Dialog::around(EditView::new().with_name("power").fixed_width(5).align_center())
                                .title("Power (Watts)"),
                        ).align_center(),
                )
                .child(
                    Dialog::around(
                        EditView::new()
                            .with_name("comments")
                            .fixed_width(20)
                            .align_center(),
                    )
                    .title("Comments"),
                )
                .child(DummyView)
                .child(Button::new("Add", move |s: &mut Cursive| {
                    add_log(s, connection.clone())
                })),
        )
        .title("Add Log"),
    );
    Ok(())
}
