use std::cmp::Ordering;

use chrono::NaiveDateTime;
use cursive_table_view::TableViewItem;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum LogbookColumn {
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
pub struct Logbook {
    pub timestamp: NaiveDateTime,
    pub call: String,
    pub rsttx: String,
    pub rstrx: String,
    pub band: String,
    pub frequency: String,
    pub mode: String,
    pub comments: String,
}

#[derive(Clone, Debug)]
pub struct OperatorConfig {
    pub id: u64,
    pub name: String,
    pub call: String,
    pub grid: String,
    pub cqz: String,
    pub ituz: String,
    pub dxcc: String,
    pub cont: String,
}

#[derive(Clone, Debug)]
pub struct LogbookExt {
    pub timestamp: NaiveDateTime,
    pub call: String,
    pub rsttx: String,
    pub rstrx: String,
    pub band: String,
    pub frequency: String,
    pub mode: String,
    pub power: String,
    pub comments: String,
    pub operator: OperatorConfig,
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
