use std::ops::Index;

use calamine::{Range, Reader, open_workbook_auto};

use crate::core::GenericResult;

use super::{Cell, is_empty_row};

pub struct SheetReader {
    sheet: Range<Cell>,
    parser: Box<dyn SheetParser>,

    prev_row_id: Option<usize>,
    next_row_id: usize,
    eof_reached: bool,
}

impl SheetReader {
    pub fn new(path: &str, parser: Box<dyn SheetParser>) -> GenericResult<SheetReader> {
        let sheet_name = parser.sheet_name();
        let sheet = open_sheet(path, sheet_name)?.ok_or_else(|| format!(
            "There is no {:?} sheet in the workbook", sheet_name))?;

        Ok(SheetReader {
            sheet, parser,
            prev_row_id: None,
            next_row_id: 0,
            eof_reached: false,
        })
    }

    pub fn repeatable_table_column_titles(&self) -> bool {
        self.parser.repeatable_table_column_titles()
    }

    pub fn next_row(&mut self) -> Option<&[Cell]> {
        while self.next_row_id < self.sheet.height() {
            let row = self.sheet.index(self.next_row_id);
            if self.parser.skip_row(row) {
                self.next_row_id += 1;
                continue;
            }

            self.prev_row_id.replace(self.next_row_id);
            self.next_row_id += 1;
            return Some(row);
        }

        self.eof_reached = true;
        None
    }

    pub fn next_row_checked(&mut self) -> GenericResult<&[Cell]> {
        Ok(self.next_row().ok_or("Got an unexpected end of sheet")?)
    }

    pub fn step_back(&mut self) {
        self.next_row_id = self.prev_row_id.take().unwrap();
        self.eof_reached = false;
    }

    #[allow(dead_code)]
    pub fn skip_empty_rows(&mut self) {
        while let Some(row) = self.next_row() {
            if !is_empty_row(row) {
                self.step_back();
                break;
            }
        }
    }

    pub fn detalize_error(&self, error: &str) -> String {
        if self.next_row_id == 0 || self.eof_reached {
            error.to_owned()
        } else {
            format!("Starting from #{} row: {:?}: {}",
                    self.next_row_id, self.sheet.index(self.next_row_id - 1), error)
        }
    }
}

pub trait SheetParser {
    fn sheet_name(&self) -> &str;

    fn repeatable_table_column_titles(&self) -> bool {
        false
    }

    fn skip_row(&self, _row: &[Cell]) -> bool {
        false
    }
}

pub fn open_sheet(path: &str, name: &str) -> GenericResult<Option<Range<Cell>>> {
    let mut workbook = open_workbook_auto(path)?;
    Ok(workbook.worksheet_range(name).transpose()?)
}