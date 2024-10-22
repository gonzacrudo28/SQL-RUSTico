use crate::comandos::comandos_implementacion::Comandos;
use crate::ejecutor::ejecutor_delete::ejecutar_delete;
use crate::ejecutor::ejecutor_insert::ejecutar_insert;
use crate::ejecutor::ejecutor_select::ejecutar_select;
use crate::ejecutor::ejecutor_update::ejecutar_update;
use crate::errores::errores_implementacion::{imprimir_error, Errores};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::vec;

/// Esta funcion se encarga de que una vez recibido la consulta y la ruta al directorio donde se encuentra la tabla, procesar la misma.
/// Si la query es delete o update, la estrategia elegida para evitar cargar el archivo en memoria es ir escribiendo los cambios (ya sean con actualizaciones o con eliminaciones) en otro archivo nuevo, el cual despues reemplezara al anterior.
pub fn ejecutar_comando(comando: Comandos, path: &String) -> Result<(), Errores> {
    match &comando {
        Comandos::Insert {
            tabla: _,
            into: _,
            valores: _,
        } => ejecutar_insert(&comando, path),
        Comandos::Update {
            tabla: _,
            clausula_set: _,
            clausula_where: _,
        } => ejecutar_update(&comando, path),
        Comandos::Delete {
            tabla: _,
            clausula_where: _,
        } => ejecutar_delete(&comando, path),
        Comandos::Select {
            campos: _,
            tabla: _,
            clausula_where: _,
            clausula_order: _,
        } => ejecutar_select(&comando, path),
    }
}

/// Esta funcion recibe la ruta al directorio y el nombre de la tabla incluida en la query, y devuelve la ruta al archivo .csv que va a ser procesado.
pub fn crear_ruta(path: &String, nombre_archivo: &String) -> String {
    format!("{}/{}.csv", path, nombre_archivo)
}

/// Esta funcion recibe como parametro una ruta a un archivo y lee solamente la primera linea para obtener el nombre de las columnas de la tabla.
pub fn obtener_primera_linea(path: &String) -> Result<Vec<String>, Errores> {
    let archivo = match File::open(path) {
        Ok(file) => file,
        Err(_e) => {
            imprimir_error(Errores::Error, "Error procesando el archivo".to_string());
            return Err(Errores::Error);
        }
    };
    let lector = BufReader::new(archivo);
    let mut lineas = lector.lines();
    let primera_linea = match lineas.next() {
        Some(Ok(line)) => line,
        _ => {
            imprimir_error(
                Errores::Error,
                "Error procesando el archivo: {}".to_string(),
            );
            return Err(Errores::Error);
        }
    };
    let columnas: Vec<String> = primera_linea.split(',').map(|s| s.to_string()).collect();
    Ok(columnas)
}

/// Esta funcion se encarga de eliminar los caracteres no deseados de los parametros que puede tener la query ingresada.
/// Ejemplo:
/// Teniendo estos valores: "(111,", "6,", "'Laptop',", "3)".
/// Aplicando la funcion a cada elemento y agregandolo a una lista: [["111"], ["6"], ["Laptop"], ["3"], [""]]
pub fn limpiar_lista(linea: &str) -> Vec<String> {
    let linea_limpia = linea
        .trim()
        .trim_start_matches('(')
        .trim_start_matches('\'')
        .trim_end_matches(';')
        .trim_end_matches(',')
        .trim_end_matches('\'')
        .trim_end_matches(')');
    let elementos: Vec<String> = linea_limpia
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    juntar_words(elementos)
}

///Esta funcion recibe un arreglo cuyos elementos son las columnas del archivo y devuelve un HashMap que tiene como claves a las columnas y como valor al indice en el cual estaria en una linea del archivo.
pub fn obtener_indices_columnas(columnas: &[String]) -> HashMap<String, usize> {
    let mut resultado: HashMap<String, usize> = HashMap::new();
    for (i, columna) in columnas.iter().enumerate() {
        resultado.insert(columna.to_string(), i);
    }
    resultado
}

/// Esta funcion sirve para poder procesar consultas en las cuales se incluyan palabras compuestas, de este modo me aseguro de no perder informacion.
pub fn juntar_words(vec: Vec<String>) -> Vec<String> {
    let mut nueva: Vec<String> = vec![];
    let mut aux: Vec<String> = vec![];
    let mut centinela = false;
    for palabra in vec {
        if palabra.starts_with("'") && palabra.ends_with("'") {
            nueva.push(
                palabra
                    .trim_start_matches("'")
                    .trim_end_matches("'")
                    .to_string(),
            );
        } else if palabra.starts_with("'") {
            aux.push(palabra);
            centinela = true
        } else if palabra.ends_with("'") {
            aux.push(palabra);
            nueva.push(aux.join(" "));
            aux.clear();
            centinela = false;
        } else if centinela {
            aux.push(format!(" {}", palabra));
        } else {
            nueva.push(palabra);
        }
    }
    nueva
}

pub fn adaptar_where(query: &str) -> Vec<String> {
    agrupar_expresiones(query)
}

fn agrupar_expresiones(query: &str) -> Vec<String> {
    let mut resultado = Vec::new();
    let mut token = String::new();
    let mut en_comillas = false;

    let chars: Vec<char> = query.chars().collect();
    let mut i: usize = 0;

    while i < chars.len() {
        let c: char = chars[i];

        if c == '\'' {
            en_comillas = !en_comillas;
            token.push(c);
        } else if en_comillas {
            token.push(c);
        } else if c.is_whitespace() {
            if !token.is_empty() {
                resultado.push(token.clone());
                token.clear();
            }
        } else if c == '(' || c == ')' {
            if !token.is_empty() {
                resultado.push(token.clone());
                token.clear();
            }
            resultado.push(c.to_string());
        } else if c == '>' || c == '<' || c == '=' {
            if !token.is_empty() {
                resultado.push(token.clone());
                token.clear();
            }
            token.push(c);
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                token.push(chars[i + 1]);
                i += 1;
            }
            resultado.push(token.clone());
            token.clear();
        } else {
            token.push(c);
        }

        i += 1;
    }

    if !token.is_empty() {
        resultado.push(token);
    }

    agrupar_condicionales(resultado)
}

fn agrupar_condicionales(tokens: Vec<String>) -> Vec<String> {
    let mut resultado = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if i + 2 < tokens.len()
            && (tokens[i + 1] == ">"
                || tokens[i + 1] == "<"
                || tokens[i + 1] == "="
                || tokens[i + 1] == ">="
                || tokens[i + 1] == "<=")
        {
            let condicional = format!("{} {} {}", tokens[i], tokens[i + 1], tokens[i + 2]);
            resultado.push(condicional);
            i += 3;
        } else {
            resultado.push(tokens[i].clone());
            i += 1;
        }
    }

    resultado
}
#[cfg(test)]
mod test {
    use crate::comandos::comandos_implementacion::Comandos;
    use crate::ejecutor::ejecutor_implementacion::ejecutar_comando;
    use crate::errores::errores_implementacion::Errores;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    #[test]
    fn test_insert_valida() {
        let ruta_tabla: String = "src".to_string();
        let query: Comandos = Comandos::Insert {
            tabla: "ordenes".to_string(),
            into: vec![
                "(id,".to_string(),
                "id_cliente,".to_string(),
                "producto,".to_string(),
                "cantidad)".to_string(),
            ],
            valores: vec![
                "(111,".to_string(),
                "6,".to_string(),
                "'Laptop',".to_string(),
                "3)".to_string(),
            ],
        };
        if let Err(_e) = ejecutar_comando(query, &ruta_tabla) {
            panic!("FALLO TEST INSERT")
        }
        let linea_esta = buscar_linea("111,6,Laptop,3".to_string(), "src/ordenes.csv".to_string());
        match linea_esta {
            Ok(bool) => assert_eq!(bool, true),
            _ => panic!("FALLO TEST INSERT"),
        }
    }
    #[test]
    fn test_insert_invalida() {
        let ruta_tabla: String = "src".to_string();
        let query: Comandos = Comandos::Insert {
            tabla: "ordenes".to_string(),
            into: vec![
                "(id,".to_string(),
                "id_cliente,".to_string(),
                "hola,".to_string(),
                "cantidad)".to_string(),
            ],
            valores: vec![
                "(111,".to_string(),
                "6,".to_string(),
                "'Laptop',".to_string(),
                "3)".to_string(),
            ],
        };
        match ejecutar_comando(query, &ruta_tabla) {
            Err(e) => assert_eq!(e, Errores::Error),
            _ => panic!("FALLO TEST INSERT"),
        }
    }

    #[test]
    fn test_delete_valida() {
        let ruta_tabla: String = "src".to_string();
        let query: Comandos = Comandos::Delete {
            tabla: "clientes".to_string(),
            clausula_where: "apellido = 'López' and email = 'ana.lopez@email.com'".to_string(),
        };
        if let Err(_e) = ejecutar_comando(query, &ruta_tabla) {
            panic!("FALLO TEST DELETE")
        }
        let linea_esta = buscar_linea(
            "2,Ana,López,ana.lopez@email.com".to_string(),
            "src/clientes.csv".to_string(),
        );
        match linea_esta {
            Ok(bool) => assert_eq!(bool, false),
            _ => panic!("FALLO TEST DELETE"),
        }
    }
    #[test]
    fn test_delete_invalida() {
        let ruta_tabla: String = "src".to_string();
        let query: Comandos = Comandos::Delete {
            tabla: "cliente".to_string(),
            clausula_where: "apellido = 'López' and email = 'ana.lopez@email.com'".to_string(),
        };
        match ejecutar_comando(query, &ruta_tabla) {
            Err(e) => assert_eq!(e, Errores::Error),
            _ => panic!("FALLO TEST DELETE"),
        }
    }
    #[test]
    fn test_update_valida() {
        let ruta_tabla: String = "src".to_string();
        let query: Comandos = Comandos::Update {
            tabla: "clientes".to_string(),
            clausula_set: "email = 'mrodriguez@hotmail.com'".to_string(),
            clausula_where: "id = 4".to_string(),
        };
        if let Err(_e) = ejecutar_comando(query, &ruta_tabla) {
            panic!("FALLO TEST UPDATE")
        }
        let linea_esta = buscar_linea(
            "4,María,Rodríguez,mrodriguez@hotmail.com".to_string(),
            "src/clientes.csv".to_string(),
        );
        match linea_esta {
            Ok(bool) => assert_eq!(bool, true),
            _ => panic!("FALLO TEST UPDATE"),
        }
    }

    pub fn buscar_linea(buscada: String, ruta: String) -> Result<bool, Errores> {
        let tabla = match File::open(ruta) {
            Ok(f) => f,
            _ => return Err(Errores::Error),
        };
        let reader = BufReader::new(tabla);
        let mut encontrada: bool = false;
        for linea in reader.lines() {
            let linea = match linea {
                Ok(l) => l,
                _ => {
                    return Err(Errores::Error);
                }
            };
            if linea == buscada {
                encontrada = true;
                break;
            }
        }
        Ok(encontrada)
    }
}
