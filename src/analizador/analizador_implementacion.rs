use std::collections::HashMap;

use crate::analizador::condicional::Condicional;
use crate::analizador::expresion::Expresion;
use crate::errores::errores_implementacion::{imprimir_error, Errores};

/// Parsea la cláusula WHERE y devuelve una expresión que la representa.
pub fn parsear_expresion(clausula_where: Vec<String>) -> Result<Expresion, Errores> {
    if clausula_where.is_empty() {
        return Ok(Expresion::Unknown);
    }
    let adaptada: Vec<Expresion> = match adaptar_where(clausula_where) {
        Err(e) => return Err(e),
        Ok(adap) => adap,
    };
    let a = match obtener_subexpresiones(adaptada) {
        Ok(c_w) => c_w,
        _ => return Err(Errores::InvalidSyntax),
    };
    Ok(a)
}

/// Esta funciom recibe un vector de strings que representa la cláusula WHERE y devuelve un vector de expresiones.
fn adaptar_where(clausula_where: Vec<String>) -> Result<Vec<Expresion>, Errores> {
    let mut expresiones: Vec<Expresion> = Vec::new();
    for actual in clausula_where {
        match actual.to_uppercase().as_str() {
            "NOT" => expresiones.push(Expresion::Not {
                derecha: Box::new(Expresion::Unknown),
            }),
            "AND" => expresiones.push(Expresion::And {
                izquierda: Box::new(Expresion::Unknown),
                derecha: Box::new(Expresion::Unknown),
            }),
            "OR" => expresiones.push(Expresion::Or {
                izquierda: Box::new(Expresion::Unknown),
                derecha: Box::new(Expresion::Unknown),
            }),
            "(" => expresiones.push(Expresion::Ipar),
            ")" => expresiones.push(Expresion::Dpar),
            _ => expresiones.push(Expresion::Operacion {
                condicional: parsear_condicional(actual.split_whitespace().collect())?,
            }),
        }
    }
    Ok(expresiones)
}

/// Parsea una condicional y devuelve un enum que la representa.
fn parsear_condicional(condicional: Vec<&str>) -> Result<Condicional, Errores> {
    if condicional.len() < 3 {
        imprimir_error(
            Errores::InvalidSyntax,
            "Error en la cláusula WHERE".to_string(),
        );
        return Err(Errores::InvalidSyntax);
    }

    let miembro1 = condicional[0].to_string();
    let operador = condicional[1];
    let miembro2 = if condicional.len() > 3 {
        condicional[2..].join(" ")
    } else {
        condicional[2].to_string()
    };

    match operador {
        ">" => Ok(Condicional::Mayor { miembro1, miembro2 }),
        ">=" => Ok(Condicional::MayorIgual { miembro1, miembro2 }),
        "<" => Ok(Condicional::Menor { miembro1, miembro2 }),
        "<=" => Ok(Condicional::MenorIgual { miembro1, miembro2 }),
        "=" => Ok(Condicional::Igual { miembro1, miembro2 }),
        _ => {
            imprimir_error(
                Errores::InvalidSyntax,
                "Operador desconocido en la cláusula WHERE".to_string(),
            );
            Err(Errores::InvalidSyntax)
        }
    }
}

/// Recibe un vector de expresiones y devuelve una expresión que las agrupa.
fn obtener_subexpresiones(exp: Vec<Expresion>) -> Result<Expresion, Errores> {
    let mut stack: Vec<Vec<Expresion>> = Vec::new();
    let mut auxiliar: Vec<Expresion> = Vec::new();
    for expresion in exp {
        match expresion {
            Expresion::Ipar => {
                stack.push(auxiliar);
                auxiliar = Vec::new();
            }
            Expresion::Dpar => {
                if let Some(mut tope) = stack.pop() {
                    let agrupada = match agrupar_expresion(&mut auxiliar) {
                        Ok(a) => a,
                        Err(e) => return Err(e),
                    };
                    tope.push(agrupada);
                    auxiliar = tope;
                } else {
                    imprimir_error(
                        Errores::InvalidSyntax,
                        "Error en paréntesis cláusula WHERE".to_string(),
                    );
                    return Err(Errores::InvalidSyntax);
                }
            }
            _ => auxiliar.push(expresion),
        }
    }
    if !stack.is_empty() {
        imprimir_error(
            Errores::InvalidSyntax,
            "Error en sintaxis clausula where".to_string(),
        );
        return Err(Errores::InvalidSyntax);
    }
    agrupar_expresion(&mut auxiliar)
}

//Precedencia: Not -> And -> Or
/// Agrupa las expresiones de una cláusula WHERE dejandola en una unica expresion que conteiene a todas.
fn agrupar_expresion(exp: &mut Vec<Expresion>) -> Result<Expresion, Errores> {
    let mut indice = 0;
    while indice < exp.len() {
        if let Expresion::Not { derecha } = &exp[indice] {
            if let Expresion::Unknown = &**derecha {
                if indice + 1 < exp.len() {
                    let derecha = Box::new(exp.remove(indice + 1));
                    exp[indice] = Expresion::Not { derecha };
                } else {
                    imprimir_error(
                        Errores::InvalidSyntax,
                        "Sintaxis incorrecta clausula where".to_string(),
                    );
                    return Err(Errores::InvalidSyntax);
                }
            }
        }
        indice += 1;
    }
    indice = 0;
    while indice < exp.len() {
        if let Expresion::And { izquierda, derecha } = &exp[indice] {
            if let (Expresion::Unknown, Expresion::Unknown) = (&**izquierda, &**derecha) {
                if indice > 0 && indice + 1 < exp.len() {
                    let izquierda = Box::new(exp.remove(indice - 1));
                    let derecha = Box::new(exp.remove(indice));
                    exp[indice - 1] = Expresion::And { izquierda, derecha };
                } else {
                    imprimir_error(
                        Errores::InvalidSyntax,
                        "Sintaxis incorrecta clausula where".to_string(),
                    );
                    return Err(Errores::InvalidSyntax);
                }
            }
        }
        indice += 1;
    }
    indice = 0;
    while indice < exp.len() {
        if let Expresion::Or { izquierda, derecha } = &exp[indice] {
            if let (Expresion::Unknown, Expresion::Unknown) = (&**izquierda, &**derecha) {
                if indice > 0 && indice + 1 < exp.len() {
                    let izquierda = Box::new(exp.remove(indice - 1));
                    let derecha = Box::new(exp.remove(indice));
                    exp[indice - 1] = Expresion::Or { izquierda, derecha };
                } else {
                    imprimir_error(
                        Errores::InvalidSyntax,
                        "Sintaxis incorrecta clausula where".to_string(),
                    );
                    return Err(Errores::InvalidSyntax);
                }
            }
        }
        indice += 1;
    }
    if exp.len() == 1 {
        let elemento = exp.remove(0);
        Ok(elemento)
    } else {
        imprimir_error(
            Errores::InvalidSyntax,
            "Sintaxis incorrecta clausula where".to_string(),
        );
        Err(Errores::InvalidSyntax)
    }
}

/// Evalúa si una línea cumple con la cláusula WHERE.
pub fn cumple_c_w(
    linea: &str,
    clausula_where: &Expresion,
    indice_columnas: &HashMap<String, usize>,
) -> Result<bool, Errores> {
    if let Expresion::Unknown = clausula_where {
        return Ok(true);
    }
    let linea: Vec<String> = linea.split(',').map(|s| s.trim().to_string()).collect();
    let cumple = evaluar_expresion(clausula_where, &linea, indice_columnas);
    Ok(cumple)
}

/// Evalúa una expresión.
fn evaluar_expresion(
    expresion: &Expresion,
    linea: &Vec<String>,
    indice_columnas: &HashMap<String, usize>,
) -> bool {
    match expresion {
        Expresion::Not { derecha } => !evaluar_expresion(derecha, linea, indice_columnas),
        Expresion::And { izquierda, derecha } => {
            evaluar_expresion(izquierda, linea, indice_columnas)
                && evaluar_expresion(derecha, linea, indice_columnas)
        }
        Expresion::Or { izquierda, derecha } => {
            evaluar_expresion(izquierda, linea, indice_columnas)
                || evaluar_expresion(derecha, linea, indice_columnas)
        }
        Expresion::Operacion { condicional } => {
            evaluar_condicional(condicional, linea, indice_columnas)
        }
        _ => false,
    }
}

/// Evalúa un condicional.
fn evaluar_condicional(
    condicional: &Condicional,
    linea: &[String],
    indice_columnas: &HashMap<String, usize>,
) -> bool {
    use Condicional::*;

    match condicional {
        Mayor { miembro1, miembro2 } => {
            comparar_valores(miembro1, miembro2, linea, indice_columnas, |a, b| a > b)
        }
        MayorIgual { miembro1, miembro2 } => {
            comparar_valores(miembro1, miembro2, linea, indice_columnas, |a, b| a >= b)
        }
        Menor { miembro1, miembro2 } => {
            comparar_valores(miembro1, miembro2, linea, indice_columnas, |a, b| a < b)
        }
        MenorIgual { miembro1, miembro2 } => {
            comparar_valores(miembro1, miembro2, linea, indice_columnas, |a, b| a <= b)
        }
        Igual { miembro1, miembro2 } => {
            comparar_valores(miembro1, miembro2, linea, indice_columnas, |a, b| a == b)
        }
    }
}

/// Compara dos valores.
fn comparar_valores<F>(
    miembro1: &str,
    miembro2: &str,
    linea: &[String],
    indice_columnas: &HashMap<String, usize>,
    comparador: F,
) -> bool
where
    F: Fn(&str, &str) -> bool,
{
    if indice_columnas.contains_key(miembro1) && indice_columnas.contains_key(miembro2) {
        let valor1 = obtener_valor(miembro1, linea, indice_columnas);
        let valor2 = obtener_valor(miembro2, linea, indice_columnas);
        match (valor1, valor2) {
            (Some(v1), Some(v2)) => return comparador(&v1, &v2),
            _ => return false,
        }
    } else if !indice_columnas.contains_key(miembro1) && indice_columnas.contains_key(miembro2) {
        let valor2 = obtener_valor(miembro2, linea, indice_columnas);
        if es_numero(miembro1) {
            match (miembro1, valor2) {
                (miembro1, Some(v2)) => return comparador(miembro1, v2.as_str()),
                _ => return false,
            }
        } else if miembro1.starts_with('\'') && miembro1.ends_with('\'') {
            let valor1 = miembro1.trim_start_matches('\'').trim_end_matches('\'');
            match (valor1, valor2) {
                (valor1, Some(valor2)) => return comparador(valor1, valor2.as_str()),
                _ => return false,
            }
        }
        return false;
    } else if indice_columnas.contains_key(miembro1) && !indice_columnas.contains_key(miembro2) {
        let valor1 = obtener_valor(miembro1, linea, indice_columnas);
        if es_numero(miembro2) {
            match (valor1, miembro2) {
                (Some(v1), miembro2) => return comparador(v1.as_str(), miembro2),
                _ => return false,
            }
        } else if miembro2.starts_with('\'') && miembro2.ends_with('\'') {
            let valor2 = miembro2.trim_start_matches('\'').trim_end_matches('\'');
            match (valor1, valor2) {
                (Some(v1), valor2) => return comparador(v1.as_str(), valor2),
                _ => return false,
            }
        }
    }
    false
}

/// Obtiene el valor de la columna en la linea actual.
fn obtener_valor(
    miembro: &str,
    linea: &[String],
    indice_columnas: &HashMap<String, usize>,
) -> Option<String> {
    if let Some(&indice) = indice_columnas.get(miembro) {
        if indice < linea.len() {
            return Some(linea[indice].clone());
        }
    }
    None
}

/// Verifica si una cadena es un número.
fn es_numero(cadena: &str) -> bool {
    cadena.chars().all(|c| c.is_ascii_digit())
}
