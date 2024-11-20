pub struct Interpreter<'a> {
    data: &'a [u8],
    current: usize,
    values: Vec<Value>,
}

impl<'a> Interpreter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Interpreter {
            data,
            current: 0,
            values: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Vec<Value> {
        self.scan_values();
        self.values
    }

    fn peek(&self) -> Option<u8> {
        self.data.get(self.current).copied()
    }

    fn addvance(&mut self) -> Option<u8> {
        let o = self.peek();
        if o.is_some() {
            self.current += 1;
        }
        o
    }

    fn scan_values(&mut self) {
        if let Some(c) = self.peek() {
            match c {
                b'+' => {
                    self.addvance();
                    let value = self.simple_string();
                    self.values.push(value);
                }
                b'*' => {
                    self.addvance();
                    let value = self.array();
                    self.values.push(value);
                }
                b'$' => {
                    self.addvance();
                    let value = self.bluk_string();
                    self.values.push(value);
                }
                _ => {}
            }
        }
    }

    fn word(&mut self) -> Vec<u8> {
        let mut chars = Vec::new();
        while let Some(c) = self.addvance() {
            match c {
                x if x != b'\r' => {
                    chars.push(x);
                }
                _ => {
                    self.addvance();
                    break;
                }
            }
        }
        chars
    }

    fn simple_string(&mut self) -> Value {
        Value::String(String::from_utf8(self.word()).unwrap())
    }

    fn bluk_string(&mut self) -> Value {
        let size = String::from_utf8(self.word())
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let mut values = Vec::new();
        for _ in 0..size {
            values.push(self.addvance().unwrap());
        }
        self.addvance();
        self.addvance();
        Value::String(String::from_utf8(values).unwrap())
    }

    fn array(&mut self) -> Value {
        let size = String::from_utf8(self.word())
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let mut values = Vec::new();
        for _ in 0..size {
            if let Some(c) = self.peek() {
                match c {
                    b'+' => {
                        self.addvance();
                        values.push(self.simple_string());
                    }
                    b'*' => {
                        self.addvance();
                        values.push(self.array());
                    }
                    b'$' => {
                        self.addvance();
                        values.push(self.bluk_string());
                    }
                    _ => {}
                }
            }
        }
        Value::Array(values)
    }
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Array(Vec<Value>),
}
