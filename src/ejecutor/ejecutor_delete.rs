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
pub fn ejecutar_delete(comando: &Comandos, path: &String) -> Result<(), Errores> {
    let (tabla, clausula_where) = match comando {
        Comandos::Delete {
            tabla,
            clausula_where,
        } => (tabla, clausula_where),
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
    procesar_archivo_delete(&ruta_tabla, adaptada, path)
}

///Esta funcion se encarga de leer el archivo, a medida que lo va leyendo, si encuentra una linea que debe ser eliminada no la escribe en un archivo auxiliar previamente creado; si no debe ser eliminada, la escribe. Finalmente hace un rename del auxiliar para que pase a ser la tabla a utilizar a futuro.
fn procesar_archivo_delete(
    ruta_tabla: &String,
    clausula_where: Expresion,
    ruta_directorio: &String,
) -> Result<(), Errores> {
    let columnas: Vec<String> = match obtener_primera_linea(ruta_tabla) {
        Ok(columna) => columna,
        _ => {
            return Err(Errores::Error);
        }
    };
    if columnas.is_empty() {
        imprimir_error(Errores::InvalidTable, "La tabla es invalida".to_string());
        return Err(Errores::InvalidTable);
    }
    let indices_columnas: HashMap<String, usize> = obtener_indices_columnas(&columnas);
    let tabla = match File::open(ruta_tabla) {
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
            imprimir_error(Errores::Error, "Error cargando la tabla".to_string());
            return Err(Errores::Error);
        }
    };
    let mut primera_linea: bool = true;
    for linea in reader.lines() {
        let linea = match linea {
            Ok(l) => l,
            _ => {
                imprimir_error(Errores::Error, "Error leyendo la tabla".to_string());
                return Err(Errores::Error);
            }
        };
        if primera_linea {
            if let Err(_e) = writeln!(archivo_actualizado, "{}", columnas.join(",")) {
                imprimir_error(Errores::Error, "Error en la tabla".to_string());
                return Err(Errores::Error);
            }
            primera_linea = false;
            continue;
        }
        let cumple: bool = match cumple_c_w(&linea, &clausula_where, &indices_columnas) {
            Ok(boolean) => boolean,
            _ => return Err(Errores::InvalidSyntax),
        };
        if cumple {
            continue;
        }
        if let Err(_e) = writeln!(archivo_actualizado, "{}", linea) {
            imprimir_error(Errores::Error, "Error escribiendo el archivo".to_string());
            return Err(Errores::Error);
        }
    }
    if let Err(_e) = fs::rename(&archivo_temporal, ruta_tabla) {
        imprimir_error(Errores::Error, "Error guardando los cambios".to_string());
        if let Err(_e) = fs::remove_file(&archivo_temporal) {
            return Err(Errores::Error);
        }
        return Err(Errores::Error);
    }
    Ok(())
}
