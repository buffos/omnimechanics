use crate::material::traits::MaterialFieldStrategy;
use crate::material::types::{LocalMaterialProperties, ReferenceMaterial};

#[derive(Debug, Clone)]
pub struct MathExpressionField {
    pub expression: String,
}

impl MathExpressionField {
    pub fn new(expression: String) -> Self {
        Self { expression }
    }

    /// Evaluates the expression with given context variables.
    /// Falls back to 1.0 if expression is blank or invalid.
    pub fn eval_with_vars(
        &self,
        x: f64,
        y: f64,
        z: f64,
        e0: f64,
        nu0: f64,
        rho0: f64,
    ) -> f64 {
        let clean = self.expression.trim();
        if clean.is_empty() {
            return e0;
        }

        // Lightweight parser / evaluator
        let mut parser = SimpleParser::new(clean, x, y, z, e0, nu0, rho0);
        match parser.parse_expression() {
            Ok(val) => val,
            Err(_) => e0, // fallback
        }
    }
}

impl MaterialFieldStrategy for MathExpressionField {
    fn evaluate(&self, point: [f64; 3], reference: &ReferenceMaterial) -> LocalMaterialProperties {
        let e = self
            .eval_with_vars(
                point[0],
                point[1],
                point[2],
                reference.e0,
                reference.nu0,
                reference.rho0,
            )
            .max(1e3); // avoid non-physical zero or negative stiffness

        let nu = reference.nu0;
        let rho = reference.rho0;
        let mu = e / (2.0 * (1.0 + nu));
        let lambda = (e * nu) / ((1.0 + nu) * (1.0 - 2.0 * nu));

        LocalMaterialProperties {
            e,
            nu,
            rho,
            mu,
            lambda,
            secant_modulus: None,
            yield_stress: None,
            scalar_metric: e,
        }
    }
}

/// Simple recursive descent expression evaluator
struct SimpleParser {
    chars: Vec<char>,
    pos: usize,
    x: f64,
    y: f64,
    z: f64,
    e0: f64,
    nu0: f64,
    rho0: f64,
}

impl SimpleParser {
    fn new(input: &str, x: f64, y: f64, z: f64, e0: f64, nu0: f64, rho0: f64) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            x,
            y,
            z,
            e0,
            nu0,
            rho0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.skip_whitespace();
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.skip_whitespace();
        if self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn parse_expression(&mut self) -> Result<f64, String> {
        let mut val = self.parse_term()?;
        while let Some(c) = self.peek() {
            if c == '+' {
                self.next_char();
                val += self.parse_term()?;
            } else if c == '-' {
                self.next_char();
                val -= self.parse_term()?;
            } else {
                break;
            }
        }
        Ok(val)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut val = self.parse_factor()?;
        while let Some(c) = self.peek() {
            if c == '*' {
                self.next_char();
                val *= self.parse_factor()?;
            } else if c == '/' {
                self.next_char();
                let divisor = self.parse_factor()?;
                if divisor.abs() < 1e-14 {
                    return Ok(val * 1e3); // guard divide-by-zero
                }
                val /= divisor;
            } else {
                break;
            }
        }
        Ok(val)
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        let mut val = self.parse_primary()?;
        while let Some(c) = self.peek() {
            if c == '^' {
                self.next_char();
                let exp = self.parse_primary()?;
                val = val.powf(exp);
            } else {
                break;
            }
        }
        Ok(val)
    }

    fn parse_primary(&mut self) -> Result<f64, String> {
        self.skip_whitespace();
        let c = self.peek().ok_or("Unexpected end of input")?;

        if c == '(' {
            self.next_char();
            let val = self.parse_expression()?;
            if self.next_char() != Some(')') {
                return Err("Missing closing parenthesis".to_string());
            }
            return Ok(val);
        }

        if c == '-' {
            self.next_char();
            return Ok(-self.parse_primary()?);
        }

        if c.is_ascii_digit() || c == '.' {
            return self.parse_number();
        }

        if c.is_alphabetic() {
            let ident = self.parse_identifier()?;
            match ident.to_lowercase().as_str() {
                "x" => Ok(self.x),
                "y" => Ok(self.y),
                "z" => Ok(self.z),
                "e0" => Ok(self.e0),
                "nu0" => Ok(self.nu0),
                "rho0" => Ok(self.rho0),
                "pi" => Ok(std::f64::consts::PI),
                "sin" => {
                    self.expect('(')?;
                    let arg = self.parse_expression()?;
                    self.expect(')')?;
                    Ok(arg.sin())
                }
                "cos" => {
                    self.expect('(')?;
                    let arg = self.parse_expression()?;
                    self.expect(')')?;
                    Ok(arg.cos())
                }
                "exp" => {
                    self.expect('(')?;
                    let arg = self.parse_expression()?;
                    self.expect(')')?;
                    Ok(arg.exp())
                }
                "sqrt" => {
                    self.expect('(')?;
                    let arg = self.parse_expression()?;
                    self.expect(')')?;
                    Ok(arg.max(0.0).sqrt())
                }
                _ => Ok(1.0),
            }
        } else {
            Err(format!("Unexpected character: {}", c))
        }
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' {
                s.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        s.parse::<f64>().map_err(|e| e.to_string())
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(s)
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        let c = self.next_char().ok_or(format!("Expected '{}'", expected))?;
        if c == expected {
            Ok(())
        } else {
            Err(format!("Expected '{}', found '{}'", expected, c))
        }
    }
}
