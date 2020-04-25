//! A simple line-oriented scripting language for small computers

/// Represents our program, which has some immutable block of memory
/// containing instructions.
pub struct Program<'a> {
    data: &'a [u8],
}

/// Errors raised by our program
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Unknown,
    FunctionNotFound,
    SequenceError(usize),
}

/// Values we understand. These are calculated from expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    StringLiteral(&'a str),
    String(String),
    Vector(Vec<Value<'a>>),
    Integer(i32),
    Float(f32),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Element<'a> {
    /// Does nothing
    Nop,
    /// Marks the end of a block
    End,
    /// Followed by a name (the string)
    Function(&'a str),
    /// Followed by an expression
    Return,
    /// Literal Integer
    Integer(i32),
}

pub struct ElementIter<'a> {
    program: &'a Program<'a>,
    index: usize,
}

impl<'a> Program<'a> {
    pub(crate) const NOP_ID: u8 = 0x00;
    pub(crate) const FUNCTION_ID: u8 = 0x01;
    pub(crate) const END_ID: u8 = 0x02;
    pub(crate) const RETURN_ID: u8 = 0x03;
    pub(crate) const INTEGER1_ID: u8 = 0x04;
    pub(crate) const INTEGER2_ID: u8 = 0x05;
    pub(crate) const INTEGER3_ID: u8 = 0x06;
    pub(crate) const INTEGER4_ID: u8 = 0x07;

    pub fn new(program_data: &'a [u8]) -> Program {
        Program { data: program_data }
    }

    pub fn iter_statements(&'a self, index: usize) -> ElementIter<'a> {
        ElementIter {
            program: self,
            index,
        }
    }

    pub fn run(&self, function_name: &str) -> Result<Value, Error> {
        let mut fn_index = None;
        // Looking for a function
        for (index, statement) in self.iter_statements(0) {
            match statement {
                Element::Function(name) => {
                    if name == function_name {
                        fn_index = Some(index + 2 + name.len());
                        break;
                    }
                }
                _ => {
                    // Skip this statement
                }
            }
        }
        if let Some(index) = fn_index {
            self.run_from_index(index)
        } else {
            Err(Error::FunctionNotFound)
        }
    }

    /// Evaluate an expression at the given index.
    ///
    /// Currently only integer literals are supported. TODO:
    ///
    /// * Addition
    ///   * Integer + Integer
    ///   * Float + Float
    ///   * String + String
    /// * Subtraction
    ///   * Integer - Integer
    ///   * Float - Float
    /// * Multiplication
    ///   * Integer * Integer
    ///   * Float * Float
    ///   * String * Integer
    /// * Division
    ///   * Integer / Integer
    ///   * Float / Float
    /// * Function call
    /// * Bitwise OR (integer)
    /// * Bitwise AND (integer)
    /// * Bitwise XOR (integer)
    /// * Unary negation
    ///   * Integer
    ///   * Float
    fn evaluate_expression(&self, index: usize) -> Result<(usize, Value), Error> {
        match self.iter_statements(index).next() {
            Some((sub_index, Element::Integer(i))) => Ok((sub_index, Value::Integer(i))),
            _ => Err(Error::SequenceError(index)),
        }
    }

    /// Runs a sequence of statements (each described by a leading `Element`).
    ///
    /// TODO:
    ///
    /// * If statement
    /// * If/Else statement
    /// * If/Elseif/Else statement
    /// * Loop statement (with break)
    /// * For loop
    pub fn run_from_index(&self, index: usize) -> Result<Value, Error> {
        for (sub_index, statement) in self.iter_statements(index) {
            match statement {
                Element::Nop => {
                    // Skip this one
                }
                Element::Return => {
                    // Pop and evaluate an expression
                    let (_new_index, value) = self.evaluate_expression(sub_index + 1)?;
                    return Ok(value);
                }
                Element::End => {
                    // End of our function
                    break;
                }
                _ => {
                    // Uh oh - shouldn't find this element inside a function as a statement
                    return Err(Error::SequenceError(sub_index));
                }
            }
        }
        Ok(Value::Nil)
    }

    /// Assuming there's a string at this index, we pull an (8-bit) length, then interpret that many bytes as UTF-8.
    ///
    /// Returns None if we run out of bytes or it doesn't look like valid UTF-8.
    fn read_string(&self, index: usize) -> Option<&str> {
        self.data.get(index).and_then(|len| {
            core::str::from_utf8(&self.data[index + 1..index + 1 + usize::from(*len)]).ok()
        })
    }
}

impl<'a> Iterator for ElementIter<'a> {
    type Item = (usize, Element<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        // Find what sort of statement is next
        match self.program.data.get(self.index).cloned() {
            Some(Program::FUNCTION_ID) => {
                if let Some(name) = self.program.read_string(self.index + 1) {
                    let old_index = self.index;
                    self.index += 2 + name.len();
                    Some((old_index, Element::Function(name)))
                } else {
                    None
                }
            }
            Some(Program::NOP_ID) => {
                let old_index = self.index;
                self.index += 1;
                Some((old_index, Element::Nop))
            }
            Some(Program::END_ID) => {
                let old_index = self.index;
                self.index += 1;
                Some((old_index, Element::End))
            }
            Some(Program::RETURN_ID) => {
                let old_index = self.index;
                self.index += 1;
                Some((old_index, Element::Return))
            }
            Some(Program::INTEGER1_ID) => {
                if let Some(i) = self.program.data.get(self.index + 1) {
                    let old_index = self.index;
                    self.index += 2;
                    Some((old_index, Element::Integer(*i as i32)))
                } else {
                    None
                }
            }
            Some(Program::INTEGER2_ID) => {
                if let Some(i) = self.program.data.get(self.index + 1..self.index + 3) {
                    let old_index = self.index;
                    self.index += 3;
                    // Stored as big endian
                    let value: i32 = (i32::from(i[0]) << 8) | i32::from(i[1]);
                    Some((old_index, Element::Integer(value)))
                } else {
                    None
                }
            }
            Some(Program::INTEGER3_ID) => {
                if let Some(i) = self.program.data.get(self.index + 1..self.index + 4) {
                    let old_index = self.index;
                    self.index += 4;
                    // Stored as big endian
                    let value: i32 =
                        (i32::from(i[0]) << 16) | (i32::from(i[1]) << 8) | i32::from(i[2]);
                    Some((old_index, Element::Integer(value)))
                } else {
                    None
                }
            }
            Some(Program::INTEGER4_ID) => {
                if let Some(i) = self.program.data.get(self.index + 1..self.index + 5) {
                    let old_index = self.index;
                    self.index += 5;
                    // Stored as big endian
                    let value: i32 = (i32::from(i[0]) << 24)
                        | (i32::from(i[1]) << 16)
                        | (i32::from(i[2]) << 8)
                        | i32::from(i[3]);
                    Some((old_index, Element::Integer(value)))
                } else {
                    None
                }
            }

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_program() {
        let data = [
            Program::FUNCTION_ID,
            0x05,
            b'f',
            b'o',
            b'o',
            0xc2,
            0xa3,
            Program::RETURN_ID,
            Program::INTEGER1_ID,
            0x01,
            Program::END_ID,
        ];
        let p = Program::new(&data);
        assert_eq!(p.run("bar"), Err(Error::FunctionNotFound));
        assert_eq!(p.run("foo£"), Ok(Value::Integer(0x01)));
    }

    #[test]
    fn num_statements() {
        let data = [
            Program::FUNCTION_ID,
            0x03,
            b'f',
            b'o',
            b'o',
            Program::NOP_ID,
            Program::INTEGER2_ID,
            0x01,
            0x02,
            Program::END_ID,
        ];
        let p = Program::new(&data);
        for (idx, s) in p.iter_statements(0) {
            println!("idx={}, s={:?}", idx, s);
        }
        assert_eq!(p.iter_statements(0).count(), 4);
    }

    #[test]
    fn get_integer1() {
        let data = [Program::INTEGER1_ID, 0x03];
        let p = Program::new(&data);
        assert_eq!(
            p.iter_statements(0).next(),
            Some((0, Element::Integer(0x03)))
        );
    }

    #[test]
    fn get_integer2() {
        let data = [Program::INTEGER2_ID, 0x03, 0x04];
        let p = Program::new(&data);
        assert_eq!(
            p.iter_statements(0).next(),
            Some((0, Element::Integer(0x0304)))
        );
    }

    #[test]
    fn get_integer3() {
        let data = [Program::INTEGER3_ID, 0x03, 0x04, 0x05];
        let p = Program::new(&data);
        assert_eq!(
            p.iter_statements(0).next(),
            Some((0, Element::Integer(0x030405)))
        );
    }

    #[test]
    fn get_integer4() {
        let data = [Program::INTEGER4_ID, 0x03, 0x04, 0x05, 0x06];
        let p = Program::new(&data);
        assert_eq!(
            p.iter_statements(0).next(),
            Some((0, Element::Integer(0x03040506)))
        );
    }

    #[test]
    fn return_integer_literal() {
        let data = [
            Program::FUNCTION_ID,
            0x03,
            b'f',
            b'o',
            b'o',
            Program::RETURN_ID,
            Program::INTEGER1_ID,
            0xF0,
            Program::END_ID,
        ];
        let p = Program::new(&data);
        assert_eq!(p.run("foo"), Ok(Value::Integer(0xF0)));
    }
}
