use anyhow::{anyhow, Result};
use chrono::{NaiveDateTime, Utc};
use cursive::menu::Tree;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Button, Dialog, DummyView, EditView, LinearLayout, SelectView};
use cursive::Cursive;
use cursive::{align::HAlign, event::Key};
use cursive_aligned_view::Alignable;
use cursive_table_view::{TableView, TableViewItem};
use rand::Rng;
use rusqlite::{params, Connection, Result as SqliteResult, MAIN_DB};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::{fs, path::PathBuf};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum LogbookColumn {
    Timestamp,
    Call,
    RSTTX,
    RSTRX,
    Band,
    Frequency,
    Mode,
    Comments,
}

impl LogbookColumn {
    fn as_str(&self) -> &str {
        match *self {
            LogbookColumn::Timestamp => "Timestamp",
            LogbookColumn::Call => "Call",
            LogbookColumn::RSTTX => "RSTTX",
            LogbookColumn::RSTRX => "RSTRX",
            LogbookColumn::Band => "Band",
            LogbookColumn::Frequency => "Frequency",
            LogbookColumn::Mode => "Mode",
            LogbookColumn::Comments => "Comments",
        }
    }
}

#[derive(Clone, Debug)]
struct Logbook {
    timestamp: NaiveDateTime,
    call: String,
    rsttx: String,
    rstrx: String,
    band: String,
    frequency: String,
    mode: String,
    comments: String,
}

impl TableViewItem<LogbookColumn> for Logbook {
    fn to_column(&self, column: LogbookColumn) -> String {
        match column {
            LogbookColumn::Timestamp => self.timestamp.to_string(),
            LogbookColumn::Call => self.call.clone(),
            LogbookColumn::RSTTX => self.rsttx.clone(),
            LogbookColumn::RSTRX => self.rstrx.clone(),
            LogbookColumn::Band => self.band.clone(),
            LogbookColumn::Frequency => self.frequency.clone(),
            LogbookColumn::Mode => self.mode.clone(),
            LogbookColumn::Comments => self.comments.clone(),
        }
    }

    fn cmp(&self, other: &Self, column: LogbookColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            LogbookColumn::Timestamp => self.timestamp.cmp(&other.timestamp),
            LogbookColumn::Call => self.call.cmp(&other.call),
            LogbookColumn::RSTTX => self.rsttx.cmp(&other.rsttx),
            LogbookColumn::RSTRX => self.rstrx.cmp(&other.rstrx),
            LogbookColumn::Band => self.band.cmp(&other.band),
            LogbookColumn::Frequency => self.frequency.cmp(&other.frequency),
            LogbookColumn::Mode => self.mode.cmp(&other.mode),
            LogbookColumn::Comments => self.comments.cmp(&other.comments),
        }
    }
}

fn add_log(s: &mut Cursive, connection: Arc<Mutex<Connection>>) {
    let callsign = s
        .call_on_name("callsign", |view: &mut EditView| view.get_content())
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
    let comments = s
        .call_on_name("comments", |view: &mut EditView| view.get_content())
        .unwrap();
    if let Ok(conn) = connection.lock() {
        let mut stmt = "INSERT INTO logs (timestamp, call, rsttx, rstrx, band, frequency, mode, comments, operator_config) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
        let timestamp = Utc::now()
            .naive_utc()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let operator_config = 0;
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
                comments,
                operator_config,
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

fn new_log(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(
                    Dialog::around(
                        EditView::new()
                            .with_name("callsign")
                            .fixed_width(10)
                            .align_center(),
                    )
                    .title("Callsign"),
                )
                .child(
                    LinearLayout::horizontal()
                        .child(
                            Dialog::around(Button::new("", select_band).with_name("band"))
                                .title("Band"),
                        )
                        .child(
                            Dialog::around(EditView::new().with_name("frequency").fixed_width(10))
                                .title("Frequency"),
                        )
                        .child(
                            Dialog::around(Button::new("", select_mode).with_name("mode"))
                                .title("Mode"),
                        ),
                )
                .child(
                    LinearLayout::horizontal()
                        .child(
                            Dialog::around(EditView::new().with_name("rsttx").fixed_width(5))
                                .title("RST TX"),
                        )
                        .child(
                            Dialog::around(EditView::new().with_name("rstrx").fixed_width(5))
                                .title("RST RX"),
                        ),
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

fn make_table(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    let mut table = TableView::<Logbook, LogbookColumn>::new()
        .column(LogbookColumn::Timestamp, "Timestamp", |c| {
            c.width_percent(20)
        })
        .column(LogbookColumn::Call, "Call", |c| c.width_percent(5))
        .column(LogbookColumn::RSTTX, "RST TX", |c| c.width_percent(5))
        .column(LogbookColumn::RSTRX, "RST RX", |c| c.width_percent(5))
        .column(LogbookColumn::Band, "Band", |c| c.width_percent(5))
        .column(LogbookColumn::Frequency, "Frequency", |c| {
            c.width_percent(10)
        })
        .column(LogbookColumn::Mode, "Mode", |c| c.width_percent(5))
        .column(LogbookColumn::Comments, "Comments", |c| c.width_percent(45));
    let logs = if let Ok(connection) = connection.lock() {
        let mut stmt = connection.prepare(
            "SELECT timestamp, call, rsttx, rstrx, band, frequency, mode, comments FROM logs ORDER BY timestamp DESC",
        )?;
        let mut logs: Vec<Logbook> = Vec::new();
        let log_out = stmt.query_map((), |row| {
            Ok(Logbook {
                timestamp: row.get(0)?,
                call: row.get(1)?,
                rsttx: row.get(2)?,
                rstrx: row.get(3)?,
                band: row.get(4)?,
                frequency: row.get(5)?,
                mode: row.get(6)?,
                comments: row.get(7)?,
            })
        })?;
        for log in log_out {
            logs.push(log?);
        }
        logs
    } else {
        return Err(anyhow!("Could not lock connection"));
    };

    table.set_items(logs);
    s.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(
                    LinearLayout::vertical()
                        .child(Button::new("Filter", |s| {
                            s.add_layer(
                                Dialog::around(
                                    LinearLayout::vertical()
                                        .child(DummyView)
                                        .child(EditView::new().fixed_width(10))
                                        .child(DummyView)
                                        .child(Button::new("Submit", |s| {
                                            s.pop_layer();
                                        })),
                                )
                                .title("Filter"),
                            )
                        }))
                        .child(Button::new("Export", |s| {
                            s.add_layer(
                                Dialog::around(
                                    LinearLayout::vertical()
                                        .child(DummyView)
                                        .child(EditView::new().fixed_width(10))
                                        .child(DummyView)
                                        .child(Button::new("Submit", |s| {
                                            s.pop_layer();
                                        })),
                                )
                                .title("Export"),
                            )
                        }))
                        .align_center(),
                )
                .child(DummyView)
                .child(table.with_name("table").min_size((150, 100))),
        )
        .title("Logbook"),
    );

    Ok(())
}

fn main() -> Result<()> {
    // This doesn't work
    let target_path = PathBuf::from("~/tuilog/tuilog.db");
    let connection = Connection::open(target_path)?;
    let query = "
        CREATE TABLE IF NOT EXISTS operatorconfig (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, call TEXT, grid TEXT, cqz TEXT, ituz TEXT, dxcc TEXT, cont TEXT);
    ";
    connection.execute(query, ())?;
    let query = "
        CREATE TABLE IF NOT EXISTS logs (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp TEXT, call TEXT, rsttx TEXT, rstrx TEXT, band TEXT, frequency TEXT, mode TEXT, comments TEXT, operator_config INTEGER NOT NULL REFERENCES operatorConfig(id));
    ";
    connection.execute(query, ())?;

    let connection = Arc::new(Mutex::new(connection));

    let mut siv = cursive::default();
    siv.set_autorefresh(true);

    let new_log_conn = connection.clone();
    let logbook_conn = connection.clone();

    siv.menubar().add_subtree(
        "File",
        Tree::new()
            .leaf("New Log", move |s| {
                new_log(s, new_log_conn.clone()).unwrap()
            })
            .leaf("Logbook", move |s| {
                make_table(s, logbook_conn.clone()).unwrap()
            })
            .leaf("Options", |s| {
                s.add_layer(cursive::views::TextView::new("Options"))
            })
            .leaf("Quit", |s| s.quit()),
    );

    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
    siv.run();

    Ok(())
}
