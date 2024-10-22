#[derive(Debug)]
/// Representa los tipos de condiciones que se pueden utilizar en una consulta.
pub enum Condicional {
    /// Representa la condición de mayor que.
    Mayor { miembro1: String, miembro2: String },
    /// Representa la condición de mayor o igual que.
    MayorIgual { miembro1: String, miembro2: String },
    /// Representa la condición de menor que.
    Menor { miembro1: String, miembro2: String },
    /// Representa la condición de menor
    MenorIgual { miembro1: String, miembro2: String },
    /// Representa la condición de igualdad.
    Igual { miembro1: String, miembro2: String },
}
