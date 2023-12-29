use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};

use adif::{AdifFile, AdifHeader, AdifRecord, AdifType};
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use cursive::{
    view::{Nameable, Resizable},
    views::{Button, Dialog, DummyView, EditView, LinearLayout, TextView},
    Cursive,
};
use cursive_aligned_view::Alignable;
use cursive_table_view::TableView;
use indexmap::map::IndexMap;
use rusqlite::Connection;

use crate::models::{Logbook, LogbookColumn, LogbookExt, OperatorConfig};

fn export(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
    let start_timestamp = s
        .call_on_name("start_timestamp", |view: &mut EditView| {
            let content = view.get_content();
            if content.len() > 0 {
                return Some(NaiveDateTime::parse_from_str(
                    content.as_str(),
                    "%Y-%m-%d %H:%M:%S",
                ));
            } else {
                return None;
            }
        })
        .unwrap();
    if let Some(Err(start_err)) = start_timestamp {
        return Err(anyhow!("Could not parse start timestamp: {}", start_err));
    }
    let end_timestamp = s
        .call_on_name("end_timestamp", |view: &mut EditView| {
            let content = view.get_content();
            if content.len() > 0 {
                return Some(NaiveDateTime::parse_from_str(
                    content.as_str(),
                    "%Y-%m-%d %H:%M:%S",
                ));
            } else {
                return None;
            }
        })
        .unwrap();
    if let Some(Err(end_err)) = end_timestamp {
        return Err(anyhow!("Could not parse start timestamp: {}", end_err));
    }
    let export_path = s
        .call_on_name("export_path", |view: &mut EditView| {
            let content = view.get_content();
            if content.len() > 0 {
                return Some(content);
            } else {
                return None;
            }
        })
        .unwrap();
    let logs = if let Ok(connection) = connection.lock() {
        let mut stmt = connection.prepare(
            "SELECT timestamp, logs.call, rsttx, rstrx, band, frequency, mode, power, comments, operatorconfig.id, name, operatorconfig.call, grid, cqz, ituz, dxcc, cont  FROM logs JOIN operatorconfig ON logs.operator_config = operatorconfig.id ORDER BY timestamp DESC;",
        ).unwrap();
        let mut logs: Vec<AdifRecord> = Vec::new();
        let log_out = stmt
            .query_map((), |row| {
                Ok(LogbookExt {
                    timestamp: row.get(0)?,
                    call: row.get(1)?,
                    rsttx: row.get(2)?,
                    rstrx: row.get(3)?,
                    band: row.get(4)?,
                    frequency: row.get(5)?,
                    mode: row.get(6)?,
                    power: row.get(7)?,
                    comments: row.get(8)?,
                    operator: OperatorConfig {
                        id: row.get(9)?,
                        name: row.get(10)?,
                        call: row.get(11)?,
                        grid: row.get(12)?,
                        cqz: row.get(13)?,
                        ituz: row.get(14)?,
                        dxcc: row.get(15)?,
                        cont: row.get(16)?,
                    },
                })
            })
            .unwrap();
        for log in log_out {
            let log = log.unwrap();
            if let Some(Ok(start)) = start_timestamp {
                if log.timestamp < start {
                    continue;
                }
            }
            if let Some(Ok(end)) = end_timestamp {
                if log.timestamp > end {
                    continue;
                }
            }
            let mut map: IndexMap<&str, AdifType> = IndexMap::new();
            map.insert("CALL", AdifType::Str(log.call));
            map.insert(
                "QSO_DATE",
                AdifType::Str(log.timestamp.format("%Y%m%d").to_string()),
            );
            map.insert(
                "TIME_ON",
                AdifType::Str(log.timestamp.format("%H%M%S").to_string()),
            );
            map.insert("FREQ", AdifType::Str(log.frequency.clone()));
            map.insert("BAND", AdifType::Str(log.band.clone()));
            map.insert("FREQ_RX", AdifType::Str(log.frequency));
            map.insert("BAND_RX", AdifType::Str(log.band));
            map.insert("COMMENT", AdifType::Str(log.comments));
            if log.mode == "USB" || log.mode == "LSB" {
                map.insert("MODE", AdifType::Str("SSB".to_string()));
                map.insert("SUBMODE", AdifType::Str(log.mode));
            } else {
                map.insert("MODE", AdifType::Str(log.mode.clone()));
            }
            map.insert("MY_GRIDSQUARE", AdifType::Str(log.operator.grid));
            map.insert("STATION_CALLSIGN", AdifType::Str(log.operator.call.clone()));
            map.insert("CQZ", AdifType::Str(log.operator.cqz));
            map.insert("ITUZ", AdifType::Str(log.operator.ituz));
            map.insert("DXCC", AdifType::Str(log.operator.dxcc));
            map.insert("CONT", AdifType::Str(log.operator.cont));
            map.insert("OPERATOR", AdifType::Str(log.operator.call));
            map.insert("RST_SENT", AdifType::Str(log.rsttx));
            map.insert("RST_RCVD", AdifType::Str(log.rstrx));
            map.insert("TX_PWR", AdifType::Str(log.power));
            logs.push(map.into());
        }
        logs
    } else {
        return Err(anyhow!("Could not lock connection"));
    };
    let mut map: IndexMap<&str, AdifType> = IndexMap::new();
    map.insert("PROGRAMVERSION", AdifType::Str("1.0.0".to_string()));
    map.insert("PROGRAMID", AdifType::Str("TUILOG".to_string()));
    let header: AdifHeader = map.into();
    let file_out = AdifFile { header, body: logs };
    let mut file = File::create(
        export_path
            .ok_or(anyhow!("No export path received"))?
            .to_string(),
    )?;
    file.write(
        file_out
            .serialize()
            .map_err(|_| anyhow!("Failed to serialize data"))?
            .as_bytes(),
    )?;
    s.pop_layer();
    Ok(())
}

pub fn make_table(s: &mut Cursive, connection: Arc<Mutex<Connection>>) -> Result<()> {
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
    let export_conn = connection.clone();
    s.pop_layer();
    s.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(
                    LinearLayout::vertical()
                        .child(Button::new("Export", move |s| {
                            let export_conn = export_conn.clone();
                            s.add_layer(
                                Dialog::around(
                                    LinearLayout::vertical()
                                        .child(DummyView)
                                        .child(TextView::new("DateTime format YYYY-MM-DD HH:MM:SS"))
                                        .child(DummyView)
                                        .child(
                                            Dialog::around(
                                                EditView::new()
                                                    .with_name("start_timestamp")
                                                    .fixed_width(10)
                                                    .align_center(),
                                            )
                                            .title("Start Timestamp"),
                                        )
                                        .child(
                                            Dialog::around(
                                                EditView::new()
                                                    .with_name("end_timestamp")
                                                    .fixed_width(10)
                                                    .align_center(),
                                            )
                                            .title("End Timestamp"),
                                        )
                                        .child(
                                            Dialog::around(
                                                EditView::new()
                                                    .with_name("export_path")
                                                    .fixed_width(10)
                                                    .align_center(),
                                            )
                                            .title("File Path"),
                                        )
                                        .child(DummyView)
                                        .child(Button::new("Submit", move |s| {
                                            export(s, export_conn.clone()).unwrap()
                                        })),
                                )
                                .title("Filter"),
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
