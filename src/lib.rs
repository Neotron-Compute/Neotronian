//! A simple line-oriented scripting language for small computers

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/// Represents our program, which has some immutable block of memory
/// containing instructions.
pub struct Program<'a> {
    data: &'a [u8],
}

/// Used to build a program.
pub struct ProgramBuilder<'a> {
    data: &'a mut [u8],
    used: usize,
}

/// Errors raised by our program
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Unknown,
    FunctionNotFound,
    SequenceError(usize),
    InsufficientSpace,
    NameTooLong,
    InvalidName,
    SyntaxError,
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

/// The elements that comprise a program.
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

/// An iterator through the elements of our program.
pub struct ElementIter<'a> {
    program: &'a Program<'a>,
    index: usize,
}

// -----------------------------------------------------------------------------
// Implementations
// -----------------------------------------------------------------------------

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

impl<'a> ProgramBuilder<'a> {
    /// Construct a new program inside a given slice
    pub fn new(space: &'a mut [u8]) -> ProgramBuilder<'a> {
        ProgramBuilder {
            data: space,
            used: 0,
        }
    }

    /// Insert an element
    pub fn insert(&mut self, element: &Element) -> Result<(), Error> {
        match element {
            Element::Nop => {
                self.insert_byte(Program::NOP_ID)?;
            }
            Element::End => {
                self.insert_byte(Program::END_ID)?;
            }
            Element::Function(name) => {
                if name.len() > 255 {
                    return Err(Error::NameTooLong);
                }
                // Avoid partial writes
                if self.free() < (2 + name.len()) {
                    return Err(Error::InsufficientSpace);
                }
                self.insert_byte(Program::FUNCTION_ID)?;
                self.insert_byte(name.len() as u8)?;
                for b in name.bytes() {
                    self.insert_byte(b)?;
                }
            }
            Element::Return => {
                self.insert_byte(Program::RETURN_ID)?;
            }
            Element::Integer(i) => {
                let mut buffer = [0u8; 5];
                let encoded_integer = Self::encode_integer(&mut buffer, *i);
                if self.free() < encoded_integer.len() {
                    return Err(Error::InsufficientSpace);
                }
                for b in encoded_integer {
                    self.insert_byte(*b)?;
                }
            }
        }
        Ok(())
    }

    /// Encode an integer`
    pub fn encode_integer(buffer: &mut [u8; 5], integer: i32) -> &[u8] {
        let bytes = integer.to_be_bytes();
        if integer >= 0 {
            // Positive
            if integer < 1 << 7 {
                // i8
                buffer[0] = Program::INTEGER1_ID;
                buffer[1] = bytes[3];
                &buffer[0..2]
            } else if integer < 1 << 15 {
                // i16
                buffer[0] = Program::INTEGER2_ID;
                buffer[1] = bytes[2];
                buffer[2] = bytes[3];
                &buffer[0..3]
            } else if integer < 1 << 23 {
                // i24
                buffer[0] = Program::INTEGER3_ID;
                buffer[1] = bytes[1];
                buffer[2] = bytes[2];
                buffer[3] = bytes[3];
                &buffer[0..4]
            } else {
                // i32
                buffer[0] = Program::INTEGER4_ID;
                buffer[1] = bytes[0];
                buffer[2] = bytes[1];
                buffer[3] = bytes[2];
                buffer[4] = bytes[3];
                &buffer[0..5]
            }
        } else {
            // Negative
            if integer >= -(1 << 7) {
                // i8
                buffer[0] = Program::INTEGER1_ID;
                buffer[1] = bytes[3];
                &buffer[0..2]
            } else if integer >= -(1 << 15) {
                // i16
                buffer[0] = Program::INTEGER2_ID;
                buffer[1] = bytes[2];
                buffer[2] = bytes[3];
                &buffer[0..3]
            } else if integer >= -(1 << 23) {
                // i24
                buffer[0] = Program::INTEGER3_ID;
                buffer[1] = bytes[1];
                buffer[2] = bytes[2];
                buffer[3] = bytes[3];
                &buffer[0..4]
            } else {
                // i32
                buffer[0] = Program::INTEGER4_ID;
                buffer[1] = bytes[0];
                buffer[2] = bytes[1];
                buffer[3] = bytes[2];
                buffer[4] = bytes[3];
                &buffer[0..5]
            }
        }
    }

    /// Add a byte to the program.
    ///
    /// Returns an error if it doesn't fit.
    fn insert_byte(&mut self, value: u8) -> Result<(), Error> {
        let Some(slot) = self.data.get_mut(self.used) else {
            return Err(Error::InsufficientSpace);
        };
        *slot = value;
        self.used += 1;
        Ok(())
    }

    /// How many bytes are used?
    pub fn used(&self) -> usize {
        self.used
    }

    /// How many bytes are free?
    pub fn free(&self) -> usize {
        self.data.len() - self.used
    }
}

impl<'a> core::convert::TryFrom<&'a str> for Element<'a> {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<Element<'a>, Error> {
        if s.eq_ignore_ascii_case("return") {
            return Ok(Element::Return);
        } else if s.eq_ignore_ascii_case("end") {
            return Ok(Element::End);
        } else if s.eq_ignore_ascii_case("nop") {
            return Ok(Element::Nop);
        } else if let Ok(i) = s.parse::<i32>() {
            return Ok(Element::Integer(i));
        } else if let Some(name) = s.strip_prefix("fn ") {
            if name.is_empty() {
                return Err(Error::InvalidName);
            }
            let mut first = true;
            for ch in name.chars() {
                if first {
                    first = false;
                    if !(ch.is_alphabetic() || ch == '_') {
                        return Err(Error::InvalidName);
                    }
                } else {
                    if !(ch.is_alphanumeric() || ch == '_') {
                        return Err(Error::InvalidName);
                    }
                }
            }
            return Ok(Element::Function(name));
        }
        Err(Error::SyntaxError)
    }
}

impl<'a> core::fmt::Display for Element<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::Nop => write!(f, "nop"),
            Element::End => write!(f, "end"),
            Element::Function(name) => write!(f, "fn {name}"),
            Element::Return => write!(f, "return"),
            Element::Integer(i) => write!(f, "{i}"),
        }
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
                    Some((old_index, Element::Integer(*i as i8 as i32)))
                } else {
                    None
                }
            }
            Some(Program::INTEGER2_ID) => {
                if let Some(i) = self.program.data.get(self.index + 1..self.index + 3) {
                    let old_index = self.index;
                    self.index += 3;
                    // Stored as big endian
                    let value: i32 = ((u16::from(i[0]) << 8) | u16::from(i[1])) as i16 as i32;
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
                    let value: u32 =
                        (u32::from(i[0]) << 16) | (u32::from(i[1]) << 8) | u32::from(i[2]);
                    // Do sign extension
                    let value: i32 = if (value & 0x0080_0000) != 0 {
                        // it's negative
                        (value | 0xFF000000) as i32
                    } else {
                        value as i32
                    };
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
                    let value: i32 = ((u32::from(i[0]) << 24)
                        | (u32::from(i[1]) << 16)
                        | (u32::from(i[2]) << 8)
                        | u32::from(i[3])) as i32;
                    Some((old_index, Element::Integer(value)))
                } else {
                    None
                }
            }

            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryInto;

    #[test]
    fn element_nop() {
        assert_eq!(Ok(Element::Nop), "nop".try_into());
        assert_eq!(Element::Nop.to_string(), "nop");
    }

    #[test]
    fn element_end() {
        assert_eq!(Ok(Element::End), "end".try_into());
        assert_eq!(Element::End.to_string(), "end");
    }

    #[test]
    fn element_function() {
        assert_eq!(Ok(Element::Function("test123")), "fn test123".try_into());
        assert_eq!(
            Err::<Element, Error>(Error::InvalidName),
            "fn test123!".try_into()
        );
        assert_eq!(
            Err::<Element, Error>(Error::InvalidName),
            "fn test 123".try_into()
        );
        assert_eq!(
            Err::<Element, Error>(Error::InvalidName),
            "fn test-123".try_into()
        );
        assert_eq!(
            Err::<Element, Error>(Error::InvalidName),
            "fn 123test".try_into()
        );
        assert_eq!(Element::Function("test123").to_string(), "fn test123");
    }

    #[test]
    fn element_return() {
        assert_eq!(Ok(Element::Return), "return".try_into());
        assert_eq!(Element::Return.to_string(), "return");
    }

    #[test]
    fn element_integer() {
        assert_eq!(Ok(Element::Integer(1234)), "1234".try_into());
        assert_eq!(Element::Integer(1234).to_string(), "1234");
    }

    #[test]
    fn empty_program() {
        let mut space = [0u8; 64];
        let builder = ProgramBuilder::new(&mut space);
        assert_eq!(builder.used(), 0);
        assert_eq!(builder.free(), space.len());
    }

    #[test]
    fn insert_function() {
        let mut space = [0u8; 64];
        let mut builder = ProgramBuilder::new(&mut space);
        builder.insert(&Element::Function("foo")).unwrap();
        assert_eq!(builder.used(), 5);
        let p = Program::new(&space[0..5]);
        assert_eq!(
            p.iter_statements(0).next(),
            Some((0, Element::Function("foo")))
        );
    }

    #[test]
    fn insert_two_functions() {
        let mut space = [0u8; 64];
        let mut builder = ProgramBuilder::new(&mut space);
        builder.insert(&Element::Function("foo")).unwrap();
        builder.insert(&Element::End).unwrap();
        builder.insert(&Element::Function("test£")).unwrap();
        builder.insert(&Element::End).unwrap();
        let expected = 2 + 3 + 1 + 2 + 6 + 1;
        assert_eq!(builder.used(), expected);
        let p = Program::new(&space[0..expected]);
        let mut iter = p.iter_statements(0);
        assert_eq!(iter.next(), Some((0, Element::Function("foo"))));
        assert_eq!(iter.next(), Some((5, Element::End)));
        assert_eq!(iter.next(), Some((6, Element::Function("test£"))));
        assert_eq!(iter.next(), Some((14, Element::End)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn create_program() {
        let data = [
            Program::FUNCTION_ID,
            0x05, // 5 byte string
            b'f', // string bytes
            b'o',
            b'o',
            0xc2, // including a UTF-8 encoded £
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
    fn get_negative_ntegeri8() {
        let data = [Program::INTEGER1_ID, 0xFF];
        let p = Program::new(&data);
        assert_eq!(p.iter_statements(0).next(), Some((0, Element::Integer(-1))));
    }

    #[test]
    fn get_negative_integer16() {
        let data = [Program::INTEGER2_ID, 0xFF, 0xFE];
        let p = Program::new(&data);
        assert_eq!(p.iter_statements(0).next(), Some((0, Element::Integer(-2))));
    }

    #[test]
    fn get_negative_integer24() {
        let data = [Program::INTEGER3_ID, 0xFF, 0xFF, 0xFD];
        let p = Program::new(&data);
        assert_eq!(p.iter_statements(0).next(), Some((0, Element::Integer(-3))));
    }

    #[test]
    fn get_negative_integer32() {
        let data = [Program::INTEGER4_ID, 0xFF, 0xFF, 0xFF, 0xFC];
        let p = Program::new(&data);
        assert_eq!(p.iter_statements(0).next(), Some((0, Element::Integer(-4))));
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
            0x0F,
            Program::END_ID,
        ];
        let p = Program::new(&data);
        assert_eq!(p.run("foo"), Ok(Value::Integer(15)));
    }

    #[test]
    fn test_integer_encoding() {
        // Check all the interesting boundary conditions. Note that 2's
        // complement integers are not symmetric - an i8 runs from -128 to +127
        // and so -128 fits in an INTEGER1 while +128 requires an INTEGER2.
        static TEST_CASES: &[(usize, i32)] = &[
            // INTEGER4
            (5, i32::MIN),
            (5, i32::MIN + 1),
            (5, -(1 << 23) - 1),
            // INTEGER3
            (4, -(1 << 23)),
            (4, -(1 << 23) + 1),
            (4, -(1 << 15) - 1),
            // INTEGER2
            (3, -(1 << 15)),
            (3, -(1 << 15) + 1),
            (3, -(1 << 7) - 1),
            // INTEGER1
            (2, -(1 << 7)),
            (2, -(1 << 7) + 1),
            (2, -1),
            (2, 0),
            (2, 1),
            (2, (1 << 7) - 1),
            // INTEGER2
            (3, (1 << 7)),
            (3, (1 << 7) + 1),
            (3, (1 << 15) - 1),
            // INTEGER3
            (4, (1 << 15)),
            (4, (1 << 15) + 1),
            (4, (1 << 23) - 1),
            // INTEGER4
            (5, (1 << 23)),
            (5, (1 << 23) + 1),
            (5, i32::MAX - 1),
            (5, i32::MAX),
        ];
        for (len, integer) in TEST_CASES {
            let mut buffer = [0u8; 5];
            let result = ProgramBuilder::encode_integer(&mut buffer, *integer);
            assert_eq!(
                *len,
                result.len(),
                "length {} != {} for {} ({:x?})",
                *len,
                result.len(),
                *integer,
                result
            );
            let p = Program::new(&result);
            assert_eq!(
                p.iter_statements(0).next(),
                Some((0, Element::Integer(*integer)))
            );
        }
    }
}

// -----------------------------------------------------------------------------
// End of file
// -----------------------------------------------------------------------------
