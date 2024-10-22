use crate::errores::errores_implementacion::{imprimir_error, Errores};

#[derive(Debug)]
/// Representa los diferentes tipos de comandos posibles que el programa soporta.
pub enum Comandos {
    /// Comando Insert.
    Insert {
        tabla: String,
        into: Vec<String>,
        valores: Vec<String>,
    },
    /// Comando Update.
    Update {
        tabla: String,
        clausula_set: String,
        clausula_where: String,
    },
    /// Comando Delete.
    Delete {
        tabla: String,
        clausula_where: String,
    },
    /// Comando Select.
    Select {
        campos: Vec<String>,
        tabla: String,
        clausula_where: String,
        clausula_order: Vec<String>,
    },
}

///Esta funcion recibe el comando tal y como es ingresado para que, segun dependiendo de la primera palabra sea parseada de una u otra manera.
pub fn parsear(comando: &str) -> Result<Comandos, Errores> {
    let token: Vec<&str> = comando.split_whitespace().collect();

    if token.is_empty() {
        imprimir_error(Errores::InvalidSyntax, "No se insertó nada".to_string());
        return Err(Errores::InvalidSyntax);
    }

    match token[0].to_uppercase().as_str() {
        "INSERT" => parser_insert(&token),
        "UPDATE" => parser_update(&token),
        "DELETE" => parser_delete(&token),
        "SELECT" => parser_select(&token),
        _ => {
            imprimir_error(Errores::InvalidSyntax, "Comando inválido".to_string());
            Err(Errores::InvalidSyntax)
        }
    }
}

/// Esta funcion parsea a las consultas de tipo insert.
fn parser_insert(token: &[&str]) -> Result<Comandos, Errores> {
    let indice_into: Option<usize> = obtener_indice(token, "INTO");
    let indice_values: Option<usize> = obtener_indice(token, "VALUES");

    if let (Some(indice_into), Some(indice_values)) = (indice_into, indice_values) {
        if indice_into != 1 || indice_values <= indice_into + 2 {
            imprimir_error(
                Errores::InvalidSyntax,
                "Los argumentos de la instrucción INSERT fueron escritos de manera incorrecta"
                    .to_string(),
            );
            return Err(Errores::InvalidSyntax);
        }
        let tabla = token[indice_into + 1].to_string();
        let valores: Vec<String> = token[indice_values + 1..]
            .iter()
            .map(|&s| s.to_string())
            .collect();
        let into: Vec<String> = token[indice_into + 2..indice_values]
            .iter()
            .map(|&s| s.to_string())
            .collect();
        let into_joined = into.join(" ");
        let into_trimmed = into_joined.trim_start_matches("(").trim_end_matches(")");
        let into_splitteada = into_trimmed.split(',').collect::<Vec<&str>>();
        let into_final: Vec<String> = into_splitteada
            .iter()
            .map(|&s| s.trim_end_matches(", ").trim().to_string())
            .collect();
        Ok(Comandos::Insert {
            tabla,
            into: into_final,
            valores,
        })
    } else {
        imprimir_error(
            Errores::InvalidSyntax,
            "Los argumentos de la instrucción INSERT fueron escritos de manera incorrecta"
                .to_string(),
        );
        Err(Errores::InvalidSyntax)
    }
}

/// Esta funcion parsea a las consultas de tipo update
fn parser_update(token: &[&str]) -> Result<Comandos, Errores> {
    let indice_set = obtener_indice(token, "SET");
    let indice_where = obtener_indice(token, "WHERE");
    let mut clausula_where: String = "".to_string();
    let mut clausula_set: String = "".to_string();
    let indice_set = match indice_set {
        Some(index) => index,
        None => {
            imprimir_error(
                Errores::InvalidSyntax,
                "Los argumentos de la instruccion UPDATE fueron escritos de manera incorrecta1"
                    .to_string(),
            );
            return Err(Errores::InvalidSyntax);
        }
    };
    let indice_where = indice_where.unwrap_or_default();
    if indice_where == 0 {
        clausula_where = "".to_string();
        clausula_set = token[indice_set + 1..].join(" ");
    }
    if indice_where != 0
        && (indice_set >= indice_where
            || indice_set + 1 >= token.len()
            || indice_where + 1 >= token.len())
    {
        imprimir_error(
            Errores::InvalidSyntax,
            "Los argumentos de la instruccion UPDATE fueron escritos de manera incorrecta"
                .to_string(),
        );
        return Err(Errores::InvalidSyntax);
    }
    let tabla: String = token[1].to_string();

    if indice_where != 0 {
        clausula_where = token[indice_where + 1..].join(" ");
        clausula_set = token[indice_set + 1..indice_where].join(" ");
    }
    Ok(Comandos::Update {
        tabla,
        clausula_set,
        clausula_where,
    })
}

/// Esta funcion parsea a las consultas de tipo delete
fn parser_delete(token: &[&str]) -> Result<Comandos, Errores> {
    if token.len() < 3 || token[1].to_uppercase() != "FROM" {
        imprimir_error(
            Errores::InvalidSyntax,
            "Los argumentos de la instruccion DELETE fueron escritos de manera incorrecta"
                .to_string(),
        );
        return Err(Errores::InvalidSyntax);
    }
    let indice_where = obtener_indice(token, "WHERE");
    let (indice_where, tiene_where) = match indice_where {
        Some(index) => (index, true),
        None => (token.len(), false),
    };
    let tabla = token[2]
        .trim_end_matches(';')
        .trim_end_matches(" ;")
        .to_string();
    let clausula_where = if tiene_where {
        if indice_where + 1 > token.len() {
            imprimir_error(
                Errores::InvalidSyntax,
                "Los argumentos de la instruccion DELETE fueron escritos de manera incorrecta"
                    .to_string(),
            );
            return Err(Errores::InvalidSyntax);
        }
        token[indice_where + 1..]
            .iter()
            .map(|&s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    } else {
        String::new()
    };
    Ok(Comandos::Delete {
        tabla,
        clausula_where,
    })
}

/// Esta funcion parsea a las consultas de tipo select.
fn parser_select(token: &[&str]) -> Result<Comandos, Errores> {
    let indice_from: Option<usize> = obtener_indice(token, "FROM");
    let indice_where: Option<usize> = obtener_indice(token, "WHERE");
    let indice_order_by: Option<usize> = obtener_indice(token, "ORDER");
    let indice_from: usize = match indice_from {
        Some(index) => index,
        None => {
            imprimir_error(
                Errores::InvalidSyntax,
                "Los argumentos de la instrucción SELECT fueron escritos de manera incorrecta"
                    .to_string(),
            );
            return Err(Errores::InvalidSyntax);
        }
    };
    let mut campos: Vec<String> = token[1..indice_from]
        .iter()
        .map(|s| s.trim_start_matches(" ").trim_end_matches(",").to_string())
        .collect();
    campos.retain(|s| !s.trim().is_empty());
    if indice_from + 1 >= token.len() {
        imprimir_error(
            Errores::InvalidSyntax,
            "Los argumentos de la instrucción SELECT fueron escritos de manera incorrecta"
                .to_string(),
        );
        return Err(Errores::InvalidSyntax);
    }

    let tabla = token[indice_from + 1].to_string();
    let mut clausula_where = Vec::new();
    let mut clausula_order = Vec::new();

    if let Some(indice_where) = indice_where {
        if let Some(indice_order_by) = indice_order_by {
            if indice_order_by + 2 <= token.len() {
                clausula_where = token[indice_where + 1..indice_order_by]
                    .iter()
                    .map(|&s| s.to_string())
                    .collect();
                clausula_order = eliminar_punto_y_coma(
                    token[indice_order_by + 2..]
                        .iter()
                        .map(|&s| s.to_string())
                        .collect(),
                );
            } else {
                imprimir_error(
                    Errores::InvalidSyntax,
                    "Los argumentos de la instrucción SELECT fueron escritos de manera incorrecta"
                        .to_string(),
                );
                return Err(Errores::InvalidSyntax);
            }
        } else {
            clausula_where = token[indice_where + 1..]
                .iter()
                .map(|&s| s.to_string())
                .collect();
        }
    } else if let Some(indice_order_by) = indice_order_by {
        if indice_order_by + 2 <= token.len() {
            clausula_order = token[indice_order_by + 2..]
                .iter()
                .map(|&s| s.to_string())
                .collect();
        } else {
            imprimir_error(
                Errores::InvalidSyntax,
                "Los argumentos de la instrucción SELECT fueron escritos de manera incorrecta"
                    .to_string(),
            );
            return Err(Errores::InvalidSyntax);
        }
    }
    let clausula_where: String = clausula_where.join(" ");
    Ok(Comandos::Select {
        campos,
        tabla,
        clausula_where,
        clausula_order,
    })
}

/// Esta funcion sirve para que dado un arreglo y un elemento, se encuentre el indice del mismo.
fn obtener_indice(token: &[&str], palabra: &str) -> Option<usize> {
    token.iter().position(|&t| t.to_uppercase() == palabra)
}

///Esta funcion elimina un posible ';' que puede haber al final de las consultas
fn eliminar_punto_y_coma(c_o: Vec<String>) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for elemento in c_o {
        if elemento != ";" {
            res.push(elemento)
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::comandos::comandos_implementacion::parsear;
    use crate::comandos::comandos_implementacion::Comandos;
    use crate::errores::errores_implementacion::Errores;
    #[test]
    fn test_parser_insert_query_valida() {
        let comando =
            "INSERT INTO ordenes (id, id_cliente, producto, cantidad) VALUES (111, 6, 'Laptop', 3)";
        match parsear(&comando) {
            Ok(Comandos::Insert {
                tabla,
                into,
                valores,
            }) => {
                let values_correcto: Vec<String> = vec![
                    "(111,".to_string(),
                    "6,".to_string(),
                    "'Laptop',".to_string(),
                    "3)".to_string(),
                ];
                let into_correcto: Vec<String> = vec![
                    "id".to_string(),
                    "id_cliente".to_string(),
                    "producto".to_string(),
                    "cantidad".to_string(),
                ];
                assert_eq!(tabla, "ordenes".to_string());
                assert_eq!(valores, values_correcto);
                assert_eq!(into, into_correcto);
            }
            _ => panic!("FALLO TEST INSERT"),
        }
    }
    #[test]
    fn test_parser_insert_query_invalida() {
        let comando = "INSERT INTO ordenes VALUES (111, 6, 'Laptop', 3)";
        match parsear(&comando) {
            Err(e) => assert_eq!(e, Errores::InvalidSyntax),
            _ => panic!("FALLO TEST INSERT"),
        }
    }

    #[test]
    fn test_parser_update_query_valida() {
        let comando = "UPDATE clientes
        SET email = 'pitymartinez@912.com.es'
        WHERE id = 4";
        match parsear(&comando) {
            Ok(Comandos::Update {
                tabla,
                clausula_set,
                clausula_where,
            }) => {
                let set_correcta: String = "email = 'pitymartinez@912.com.es'".to_string();
                let where_correcta: String = "id = 4".to_string();
                assert_eq!(tabla, "clientes".to_string());
                assert_eq!(clausula_set, set_correcta);
                assert_eq!(clausula_where, where_correcta);
            }
            _ => panic!("FALLO TEST UPDATE"),
        }
    }

    #[test]
    fn test_parser_update_query_invalida() {
        let comando = "UPDATE clientes
        email = 'pitymartinez@912.com.es'
        WHERE id = 4";
        match parsear(&comando) {
            Err(e) => {
                assert_eq!(e, Errores::InvalidSyntax)
            }
            _ => panic!("FALLO TEST UPDATE"),
        }
    }

    #[test]
    fn test_parser_delete_query_valida() {
        let comando = "DELETE FROM clientes
        WHERE apellido = 'López'";
        match parsear(&comando) {
            Ok(Comandos::Delete {
                tabla,
                clausula_where,
            }) => {
                let where_correcta: String = "apellido = 'López'".to_string();
                assert_eq!(tabla, "clientes".to_string());
                assert_eq!(clausula_where, where_correcta);
            }
            _ => panic!("FALLO TEST DELETE"),
        }
    }

    #[test]
    fn test_parser_delete_query_invalida() {
        let comando = "DELETE clientes
        WHERE apellido = 'López'";
        match parsear(&comando) {
            Err(e) => {
                assert_eq!(e, Errores::InvalidSyntax)
            }
            _ => panic!("FALLO TEST DELETE"),
        }
    }

    #[test]
    fn test_parser_select_query_valida() {
        let comando = "SELECT id, nombre, email
        FROM clientes *
        ORDER BY email DESC";
        match parsear(&comando) {
            Ok(Comandos::Select {
                campos,
                tabla,
                clausula_where,
                clausula_order,
            }) => {
                let campos_correcta =
                    vec!["id".to_string(), "nombre".to_string(), "email".to_string()];
                let tabla_correcta: String = "clientes".to_string();
                let where_correcta = "".to_string();
                let order_correcta: Vec<String> = vec!["email".to_string(), "DESC".to_string()];
                assert_eq!(campos, campos_correcta);
                assert_eq!(tabla, tabla_correcta);
                assert_eq!(clausula_where, where_correcta);
                assert_eq!(clausula_order, order_correcta);
            }
            _ => panic!("FALLO TEST SELECT"),
        }
    }

    #[test]
    fn test_parser_select_query_invalida() {
        let comando = "SELECT id, nombre, email
        FROM clientes *
        ORDER";
        match parsear(&comando) {
            Err(e) => {
                assert_eq!(e, Errores::InvalidSyntax)
            }
            _ => panic!("FALLO TEST SELECT"),
        }
    }
}
