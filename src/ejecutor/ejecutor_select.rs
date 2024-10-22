use crate::analizador::analizador_implementacion::{cumple_c_w, parsear_expresion};
use crate::analizador::expresion::Expresion;
use crate::comandos::comandos_implementacion::Comandos;
use crate::ejecutor::ejecutor_implementacion::{
    adaptar_where, crear_ruta, obtener_indices_columnas, obtener_primera_linea,
};
use crate::errores::errores_implementacion::{imprimir_error, Errores};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn ejecutar_select(comando: &Comandos, path: &String) -> Result<(), Errores> {
    let (campos, tabla, clausula_where, clausula_order) = match comando {
        Comandos::Select {
            campos,
            tabla,
            clausula_where,
            clausula_order,
        } => (campos, tabla, clausula_where, clausula_order),
        _ => {
            imprimir_error(
                Errores::InvalidSyntax,
                "Error en la query Select".to_string(),
            );
            return Err(Errores::InvalidSyntax);
        }
    };
    let ruta_archivo = crear_ruta(path, tabla);
    let clausula_where_limpia: Vec<String> =
        adaptar_where(clausula_where.trim_end_matches(";").trim_end_matches("; "));
    let adaptada: Expresion = match parsear_expresion(clausula_where_limpia) {
        Ok(a) => a,
        _ => return Err(Errores::InvalidSyntax),
    };
    procesar_archivo_select(&ruta_archivo, adaptada, clausula_order, campos)
}

fn procesar_archivo_select(
    ruta_archivo: &String,
    clausula_where: Expresion,
    clausula_order: &[String],
    campos: &Vec<String>,
) -> Result<(), Errores> {
    let columnas: Vec<String> = match obtener_primera_linea(ruta_archivo) {
        Ok(columna) => columna,
        _ => {
            return Err(Errores::Error);
        }
    };
    let indice_columnas = obtener_indices_columnas(&columnas);
    let mut resultado: Vec<String> = Vec::new();
    let tabla = match File::open(ruta_archivo) {
        Ok(f) => f,
        _ => {
            imprimir_error(Errores::Error, "No se pudo abrir el archivo".to_string());
            return Err(Errores::Error);
        }
    };

    let reader = BufReader::new(tabla);
    let mut primera_fila: bool = true;
    for linea in reader.lines() {
        let linea = match linea {
            Ok(l) => l,
            _ => {
                imprimir_error(Errores::Error, "Error leyendo la tabla".to_string());
                return Err(Errores::Error);
            }
        };
        if primera_fila {
            primera_fila = false;
            continue;
        }
        let cumple = match cumple_c_w(&linea, &clausula_where, &indice_columnas) {
            Ok(boolean) => boolean,
            _ => return Err(Errores::InvalidSyntax),
        };
        if cumple {
            resultado.push(linea);
        }
    }
    let resultado_ordenado = match ordenar_resultado(resultado, &indice_columnas, clausula_order) {
        Ok(r) => r,
        Err(_e) => return Err(Errores::InvalidSyntax),
    };
    mostrar_resultado(&resultado_ordenado, columnas, campos, &indice_columnas);
    Ok(())
}

fn ordenar_resultado(
    resultado: Vec<String>,
    columnas: &HashMap<String, usize>,
    clausula_order: &[String],
) -> Result<Vec<Vec<String>>, Errores> {
    let mut res: Vec<Vec<String>> = Vec::new();
    for elemento in resultado {
        let elem_spliteado: Vec<String> =
            elemento.split(',').map(|s| s.trim().to_string()).collect();
        res.push(elem_spliteado);
    }
    if clausula_order.is_empty() {
        return Ok(res);
    }
    let columna = &clausula_order[0];
    let modo_ordenamiento = if clausula_order.len() >= 2 {
        clausula_order[1]
            .to_uppercase()
            .trim_end_matches(";")
            .to_string()
    } else {
        "ASC".to_string()
    };
    if let Some(&indice_columna) = columnas.get(columna) {
        match modo_ordenamiento.as_str() {
            "ASC" => {
                res.sort_by(|a, b| a[indice_columna].trim().cmp(b[indice_columna].trim()));
            }
            "DESC" => {
                res.sort_by(|a, b| b[indice_columna].trim().cmp(a[indice_columna].trim()));
            }
            _ => return Err(Errores::Error),
        }
    } else {
        imprimir_error(
            Errores::InvalidSyntax,
            "Error en los argumentos de ORDER BY".to_string(),
        );
        return Err(Errores::InvalidSyntax);
    }

    Ok(res)
}

fn mostrar_resultado(
    res: &Vec<Vec<String>>,
    columnas: Vec<String>,
    campos: &Vec<String>,
    indice_columnas: &HashMap<String, usize>,
) {
    if campos == &vec!["*".to_string()] {
        println!("{}", columnas.join(","));
        for linea in res {
            println!("{}", linea.join(","));
        }
    } else {
        println!("{}", campos.join(","));
        for elemento in res {
            let mut fila: Vec<&str> = Vec::new();
            for campo in campos {
                if let Some(&indice) = indice_columnas.get(campo.trim()) {
                    if let Some(valor) = elemento.get(indice) {
                        fila.push(valor);
                    }
                }
            }
            println!("{}", fila.join(","));
        }
    }
}
