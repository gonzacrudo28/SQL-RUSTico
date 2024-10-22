use crate::comandos::comandos_implementacion::Comandos;
use crate::ejecutor::ejecutor_implementacion::{crear_ruta, limpiar_lista, obtener_primera_linea};
use crate::errores::errores_implementacion::{imprimir_error, Errores};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
/// Esta funcion se encarga de ejecutar la consulta.
pub fn ejecutar_insert(comando: &Comandos, path: &String) -> Result<(), Errores> {
    let (tabla, into, valores) = match comando {
        Comandos::Insert {
            tabla,
            into,
            valores,
        } => (tabla, into, valores),
        _ => {
            imprimir_error(Errores::Error, "Error procesando la consulta".to_string());
            return Err(Errores::Error);
        }
    };
    let mut valores_limpia: Vec<Vec<String>> = Vec::new();
    for valor in valores.iter() {
        let resultado: Vec<String> = limpiar_lista(valor);
        valores_limpia.push(resultado);
    }
    let valores_final: Vec<Vec<String>> = juntar_valores(valores_limpia, into.len());
    let into_final: Vec<String> = into
        .iter()
        .map(|s| {
            s.trim_start_matches('(')
                .trim_end_matches(')')
                .trim_end_matches(',')
                .to_string()
        })
        .collect();
    let ruta: String = crear_ruta(path, tabla);
    procesar_archivo_insert(&ruta, into_final, valores_final)
}

fn juntar_valores(valores_limpia: Vec<Vec<String>>, tam: usize) -> Vec<Vec<String>> {
    let mut resultado: Vec<Vec<String>> = Vec::new();
    let mut grupo_actual: Vec<String> = Vec::new();
    let mut contador = 0;
    for sub_lista in valores_limpia.iter() {
        let elemento = &sub_lista[0];
        grupo_actual.push(elemento.to_string());
        contador += 1;
        if contador % tam == 0 {
            resultado.push(grupo_actual);
            grupo_actual = Vec::new();
        }
    }

    resultado
}

fn procesar_archivo_insert(
    path: &String,
    into: Vec<String>,
    valores: Vec<Vec<String>>,
) -> Result<(), Errores> {
    let columnas: Vec<String> = match obtener_primera_linea(path) {
        Ok(columna) => columna,
        _ => {
            imprimir_error(Errores::Error, "Error leyendo el archivo".to_string());
            return Err(Errores::Error);
        }
    };

    if columnas.len() < into.len() || !misma_len(&valores) || into.len() != valores[0].len() {
        imprimir_error(
            Errores::InvalidSyntax,
            "Los campos a ingresar no son validos".to_string(),
        );
        return Err(Errores::Error);
    }

    let linea_nueva: Vec<Vec<String>> = match obtener_linea_a_escribir(columnas, into, valores) {
        Ok(linea) => linea,
        Err(_e) => return Err(Errores::Error),
    };

    let mut tiene_salto: bool = false;

    let file = match File::open(path) {
        Ok(f) => f,
        _ => {
            imprimir_error(Errores::Error, "Error leyendo el archivo".to_string());
            return Err(Errores::Error);
        }
    };
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    if reader.read_to_string(&mut buffer).is_ok() && buffer.ends_with('\n') {
        tiene_salto = true;
    }

    let mut archivo: std::fs::File = match OpenOptions::new().append(true).open(path) {
        Ok(file) => file,
        Err(_e) => {
            imprimir_error(Errores::Error, "Error escribiendo el archivo".to_string());
            return Err(Errores::Error);
        }
    };
    if !tiene_salto {
        match writeln!(archivo) {
            Ok(_) => {}
            Err(_) => {
                imprimir_error(Errores::Error, "Error escribiendo el archivo".to_string());
                return Err(Errores::Error);
            }
        }
    }
    for elemento in linea_nueva.iter() {
        if let Err(_e) = writeln!(archivo, "{}", elemento.join(",")) {
            imprimir_error(Errores::Error, "Error escribiendo el archivo".to_string());
            return Err(Errores::Error);
        }
    }

    Ok(())
}

/// Esta funcion devuelve la linea/s a insertar en la tabla.
fn obtener_linea_a_escribir(
    columnas: Vec<String>,
    into: Vec<String>,
    valores: Vec<Vec<String>>,
) -> Result<Vec<Vec<String>>, Errores> {
    let mut lineas: Vec<Vec<String>> = Vec::new();

    for valor in valores.iter() {
        let mut linea_actual: Vec<String> = vec!["".to_string(); columnas.len()];

        for (i, columna_into) in into.iter().enumerate() {
            let columna_limpia = columna_into.trim_matches(&['(', ')', ' '][..]);
            if let Some(indice) = columnas.iter().position(|x| x == columna_limpia) {
                linea_actual[indice] = valor.get(i).unwrap_or(&"".to_string()).to_string();
            } else {
                imprimir_error(Errores::InvalidSyntax, "Error en la insercion".to_string());
                return Err(Errores::InvalidSyntax);
            }
        }

        lineas.push(linea_actual);
    }

    Ok(lineas)
}

fn misma_len(lista: &[Vec<String>]) -> bool {
    if lista.is_empty() {
        return true;
    }
    let len = lista[0].len();
    for item in lista.iter() {
        if item.len() != len {
            return false;
        }
    }
    true
}
