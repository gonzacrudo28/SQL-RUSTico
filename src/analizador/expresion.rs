use crate::analizador::condicional::Condicional;
#[derive(Debug)]
/// Representa los tipos de expresiones booleanas que se pueden utilizar en una consulta.
pub enum Expresion {
    /// Representa la negación de una expresión.
    Not { derecha: Box<Expresion> },
    /// Representa la conjunción de dos expresiones.
    And {
        izquierda: Box<Expresion>,
        derecha: Box<Expresion>,
    },
    /// Representa la disyunción de dos expresiones.
    Or {
        izquierda: Box<Expresion>,
        derecha: Box<Expresion>,
    },
    /// Representa una operación condicional.
    Operacion { condicional: Condicional },
    /// Representa un elemento que todavia no tiene un valor asignado
    Unknown,
    /// Representa un parentesis izquierdo.
    Ipar,
    /// Representa un parentesis derecho.
    Dpar,
}
