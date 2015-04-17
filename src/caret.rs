/*
 * Copyright (c) 2015 Mathias Hällman
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cmp;

use buffer;

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum Adjustment {
  LineUp,
  LineDown,
  CharNext,
  CharPrev,
  Set(usize, usize),
}

/*
 * While the caret is in buffer coordinates, the saved column is in screen cell
 * coordinates.
 */
#[derive(Clone, Copy)]
pub struct Caret {
  line: usize,
  column: usize,
  saved_column: Option<usize>,
}

impl Caret {
  pub fn new() -> Caret {
    Caret { line: 0, column: 0, saved_column: None }
  }

  pub fn line(&self) -> usize {
    self.line
  }

  pub fn column(&self) -> usize {
    self.column
  }

  // some adjustments may assume that the caret is in a valid position
  pub fn adjust(&mut self, adjustment: Adjustment, buffer: &buffer::Buffer) {
    let (line, column) = (self.line, self.column);
    let (new_line, new_column, new_saved_column) = match adjustment {
      Adjustment::CharPrev          =>
        (line, cmp::max(0, column as isize - 1) as usize, None),
      Adjustment::CharNext          => {
        let line_length = buffer.line_length(line).unwrap();
        let max_column = cmp::max(0, line_length as isize - 1) as usize;
        (line, cmp::min(max_column, column + 1), None)
      }
      Adjustment::LineUp            =>
        if line == 0 { (line, column, self.saved_column) }
        else { self.vertical_caret_movement(line, line - 1, buffer) },
      Adjustment::LineDown          => {
        let max_line = cmp::max(0, buffer.num_lines() as isize - 1) as usize;
        if line == max_line { (line, column, self.saved_column) }
        else { self.vertical_caret_movement(line, line + 1, buffer) }
      }
      Adjustment::Set(line, column) => (line, column, None),
    };
    if line != new_line || column != new_column {
      self.line = new_line;
      self.column = new_column;
      self.saved_column = new_saved_column;
    }
  }

  // helper function to adjust, restricts the caret column to valid
  // character positions in screen space
  fn vertical_caret_movement(&self, from_line: usize, to_line: usize,
                             buffer: &buffer::Buffer)
      -> (usize, usize, Option<usize>) {
    let to_line_length = buffer.line_length(to_line).unwrap();
    let to_line_screen_length =
      buffer_to_screen_column(to_line, to_line_length, buffer);
    let max_column = cmp::max(0, to_line_screen_length as isize - 1) as usize;
    let desired_column = self.saved_column.unwrap_or(
      buffer_to_screen_column(from_line, self.column, buffer));
    let screen_column = cmp::min(max_column, desired_column);
    let buffer_column =
      screen_to_buffer_column(to_line, screen_column, buffer).unwrap();
    (to_line, buffer_column,
      if buffer_column == desired_column { None } else { Some(desired_column) })
  }
}

// sums up the widths of the characters before the given buffer column
pub fn buffer_to_screen_column(line: usize, column: usize,
                               buffer: &buffer::Buffer) -> usize {
  buffer.line_iter().from(line).next().map(|chars|
    chars.take(column).map(|c| c.width(false).unwrap_or(0)).sum()).
  unwrap_or(0)
}

// scans a line, counting characters up to the given screen column
pub fn screen_to_buffer_column(row: usize, screen_column: usize,
                               buffer: &buffer::Buffer) -> Option<usize> {
  buffer.line_iter().from(row).next().map(|chars|
    chars.filter(|&c| c != '\n').scan(0, |sum, c| {
      *sum += c.width(false).unwrap_or(0);
      Some(*sum) }).
    take_while(|&sum| sum <= screen_column).count())
}

#[cfg(test)]
mod test {
  use std::path::Path;

  use buffer;

  #[test]
  fn adjust() {
    let buffer = buffer::Buffer::open(
      &Path::new("tests/caret/hokey_pokey_caret.txt")).unwrap();
    let mut caret = super::Caret::new();
    // move to empty line
    caret.adjust(super::Adjustment::LineDown, &buffer);
    assert_eq!(caret.line, 1); assert_eq!(caret.column, 0);
    // move to end of double width character then back again
    caret.line = 3; caret.column = 3;
    caret.adjust(super::Adjustment::LineDown, &buffer);
    assert_eq!(caret.line, 4); assert_eq!(caret.column, 1);
    caret.adjust(super::Adjustment::LineUp, &buffer);
    assert_eq!(caret.line, 3); assert_eq!(caret.column, 3);
    // move to shorter lines then back again
    caret.line = 6; caret.column = 30;
    caret.adjust(super::Adjustment::LineDown, &buffer);
    assert_eq!(caret.line, 7); assert_eq!(caret.column, 14);
    caret.adjust(super::Adjustment::LineDown, &buffer);
    assert_eq!(caret.line, 8); assert_eq!(caret.column, 20);
    caret.adjust(super::Adjustment::LineUp, &buffer);
    caret.adjust(super::Adjustment::LineUp, &buffer);
    assert_eq!(caret.line, 6); assert_eq!(caret.column, 30);
    // move to shorter line, step sideways, then back again
    caret.line = 10; caret.column = 75;
    caret.adjust(super::Adjustment::LineDown, &buffer);
    assert_eq!(caret.line, 11); assert_eq!(caret.column, 68);
    caret.adjust(super::Adjustment::CharPrev, &buffer);
    assert_eq!(caret.line, 11); assert_eq!(caret.column, 67);
    caret.adjust(super::Adjustment::CharNext, &buffer);
    assert_eq!(caret.line, 11); assert_eq!(caret.column, 68);
    caret.adjust(super::Adjustment::LineUp, &buffer);
    assert_eq!(caret.line, 10); assert_eq!(caret.column, 68);
    // move to end of line lacking newline
    caret.line = 13; caret.column = 34;
    caret.adjust(super::Adjustment::LineDown, &buffer);
    assert_eq!(caret.line, 14); assert_eq!(caret.column, 34);
    caret.adjust(super::Adjustment::CharNext, &buffer);
    assert_eq!(caret.line, 14); assert_eq!(caret.column, 34);
    // simple set
    caret.adjust(super::Adjustment::Set(7, 3), &buffer);
    assert_eq!(caret.line, 7); assert_eq!(caret.column, 3);
  }
}