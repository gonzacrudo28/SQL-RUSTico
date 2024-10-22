#[derive(Debug, PartialEq)]

/// Representa los tipos de errores que pueden ocurrir durante la ejecucion del programa.
pub enum Errores {
    /// Tipo de error para cuando existen problemas relacionados al archivo .csv donde se encuentran los datos.
    InvalidTable,
    /// Tipo de error para cuando se intenta acceder o modificar una columna que no se encuentra en el archivo.
    InvalidColumn,
    /// Tipo de error para cuando se le pasa como parametro al programa una query invalida.
    InvalidSyntax,
    /// Tipo de error generico para fallos inesperados.
    Error,
}
/// Esta funcion se encarga de imprimir un error con una descripcion, ambos son pasados como parametro.
/// Imprime: [TIPO ERROR]: DESCRIPCION
pub fn imprimir_error(error: Errores, descripcion: String) {
    let tipo = match error {
        Errores::InvalidTable => "INVALID_TABLE",
        Errores::InvalidColumn => "INVALID_COLUMN",
        Errores::InvalidSyntax => "INVALID_SYNTAX",
        Errores::Error => "ERROR",
    };
    println!("[{}]: {}", tipo, descripcion);
}
