use crate::analizador::analizador_implementacion::{cumple_c_w, parsear_expresion};
use crate::analizador::expresion::Expresion;
use crate::comandos::comandos_implementacion::Comandos;
use crate::ejecutor::ejecutor_implementacion::{
    adaptar_where, crear_ruta, obtener_indices_columnas, obtener_primera_linea,
};
use crate::errores::errores_implementacion::{imprimir_error, Errores};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};

/// Esta funcion se encarga de ejecutar la consulta.
pub fn ejecutar_update(comando: &Comandos, path: &String) -> Result<(), Errores> {
    let (tabla, clausula_set, clausula_where) = match comando {
        Comandos::Update {
            tabla,
            clausula_set,
            clausula_where,
        } => (tabla, clausula_set, clausula_where),
        _ => {
            imprimir_error(Errores::Error, "Error procesando la consulta".to_string());
            return Err(Errores::Error);
        }
    };
    let clausula_set_limpia: Vec<Vec<String>> = match clausula_set_limpia(clausula_set) {
        Ok(lista) => lista,
        _ => {
            imprimir_error(Errores::Error, "Error procesando la consulta".to_string());
            return Err(Errores::Error);
        }
    };
    let ruta_tabla: String = crear_ruta(path, tabla);
    let clausula_where_limpia: Vec<String> =
        adaptar_where(clausula_where.trim_end_matches(";").trim_end_matches("; "));
    let adaptada: Expresion = match parsear_expresion(clausula_where_limpia) {
        Ok(a) => a,
        _ => return Err(Errores::InvalidSyntax),
    };
    procesar_archivo_update(&ruta_tabla, clausula_set_limpia, adaptada, path)
}

fn procesar_archivo_update(
    path: &String,
    clausula_set: Vec<Vec<String>>,
    clausula_where: Expresion,
    ruta_directorio: &String,
) -> Result<(), Errores> {
    let columnas: Vec<String> = match obtener_primera_linea(path) {
        Ok(columna) => columna,
        _ => {
            return Err(Errores::Error);
        }
    };
    let indices_columnas: HashMap<String, usize> = obtener_indices_columnas(&columnas);
    let sintaxis_set: bool = validar_clausula_set(&clausula_set, &columnas);
    if columnas.is_empty() {
        imprimir_error(Errores::InvalidTable, "La tabla es invalida".to_string());
        return Err(Errores::InvalidTable);
    } else if !sintaxis_set {
        imprimir_error(
            Errores::InvalidSyntax,
            "La clausula set es invalida".to_string(),
        );
        return Err(Errores::InvalidColumn);
    }
    actualizar_archivo(
        path,
        columnas,
        indices_columnas,
        clausula_set,
        clausula_where,
        ruta_directorio,
    )
}

fn actualizar_archivo(
    path: &String,
    columnas: Vec<String>,
    indice_columnas: HashMap<String, usize>,
    clausula_set: Vec<Vec<String>>,
    clausula_where: Expresion,
    ruta_directorio: &String,
) -> Result<(), Errores> {
    let tabla = match File::open(path) {
        Ok(f) => f,
        _ => {
            imprimir_error(Errores::Error, "No se pudo abrir el archivo".to_string());
            return Err(Errores::Error);
        }
    };
    let reader = BufReader::new(tabla);
    let archivo_temporal = crear_ruta(ruta_directorio, &"archivo_temporal".to_string());
    let mut archivo_actualizado = match File::create(&archivo_temporal) {
        Ok(f) => f,
        _ => {
            imprimir_error(Errores::Error, "Error actualizando la tabla".to_string());
            return Err(Errores::Error);
        }
    };
    let mut primera_linea: bool = true;
    for linea in reader.lines() {
        let mut linea = match linea {
            Ok(l) => l,
            _ => {
                imprimir_error(Errores::Error, "Error leyendo la tabla".to_string());
                return Err(Errores::Error);
            }
        };
        if primera_linea {
            if let Err(_e) = writeln!(archivo_actualizado, "{}", columnas.join(",")) {
                imprimir_error(Errores::Error, "Error escribiendo el archivo".to_string());
                return Err(Errores::Error);
            }
            primera_linea = false;
            continue;
        }
        //Tengo la exp, tengo que evaluar acá:
        let cumple = match cumple_c_w(&linea, &clausula_where, &indice_columnas) {
            Ok(boolean) => boolean,
            _ => return Err(Errores::InvalidSyntax),
        };
        if cumple {
            linea = match actualizar_linea(&linea, &clausula_set, &indice_columnas) {
                Ok(l) => l,
                _ => return Err(Errores::Error),
            };
        }
        // if cumple_con_where(&linea, &clausula_where, &indice_columnas) {
        //     linea = match actualizar_linea(&linea, &clausula_set, &indice_columnas) {
        //         Ok(l) => l,
        //         _ => return Err(Errores::Error),
        //     };
        // }

        if let Err(_e) = writeln!(archivo_actualizado, "{}", linea) {
            imprimir_error(Errores::Error, "Error escribiendo el archivo".to_string());
            return Err(Errores::Error);
        }
    }
    if let Err(_e) = fs::rename(&archivo_temporal, path) {
        imprimir_error(Errores::Error, "Error guardando los cambios".to_string());
        if let Err(_e) = fs::remove_file(&archivo_temporal) {
            return Err(Errores::Error);
        }
        return Err(Errores::Error);
    }
    Ok(())
}

fn actualizar_linea(
    linea: &str,
    clausula_set: &Vec<Vec<String>>,
    indice_columnas: &HashMap<String, usize>,
) -> Result<String, Errores> {
    let mut linea_separada: Vec<String> = linea.split(',').map(|s| s.to_string()).collect();
    for actual in clausula_set {
        let columna_a_modificar = actual[0].trim();
        let nuevo_valor = actual[1]
            .trim_start_matches(" \'")
            .trim_start_matches("\'")
            .trim_start_matches(" ")
            .trim_end_matches(" \'")
            .trim_end_matches("\'");
        if let Some(&indice) = indice_columnas.get(columna_a_modificar) {
            if indice < linea_separada.len() {
                linea_separada[indice] = nuevo_valor.to_string();
            } else {
                imprimir_error(Errores::Error, "Error actualizando valores".to_string());
                return Err(Errores::Error);
            }
        } else {
            imprimir_error(Errores::Error, "Error actualizando valores".to_string());
            return Err(Errores::Error);
        }
    }
    Ok(linea_separada.join(","))
}

fn clausula_set_limpia(clausula_set: &str) -> Result<Vec<Vec<String>>, Errores> {
    let clausula_separada: Vec<String> = clausula_set.split(",").map(|s| s.to_string()).collect();
    match obtener_clausula(clausula_separada) {
        Ok(res) => Ok(res),
        _ => Err(Errores::Error),
    }
}

fn obtener_clausula(clausula_separada: Vec<String>) -> Result<Vec<Vec<String>>, Errores> {
    let mut resultado: Vec<Vec<String>> = Vec::new();
    for item in clausula_separada.iter() {
        let actual: Vec<String> = item.split("=").map(|s| s.to_string()).collect();
        if actual.len() != 2 {
            imprimir_error(
                Errores::InvalidSyntax,
                "Error en la clausula set de la instruccion update".to_string(),
            );
            return Err(Errores::InvalidSyntax);
        }
        resultado.push(actual);
    }
    Ok(resultado)
}

fn validar_clausula_set(clausula_set: &[Vec<String>], columnas: &[String]) -> bool {
    if clausula_set.len() > columnas.len() {
        return false;
    }
    let mut auxiliar: Vec<&String> = Vec::new();
    for i in 0..columnas.len() {
        if i >= clausula_set.len() {
            return true;
        }
        let actual = clausula_set[i][0]
            .trim_end_matches(" ")
            .trim_start_matches(" ");
        if columnas.contains(&actual.to_string()) && !auxiliar.contains(&&clausula_set[i][0]) {
            auxiliar.push(&clausula_set[i][0]);
            continue;
        } else {
            return false;
        }
    }
    true
}
