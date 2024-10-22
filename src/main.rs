mod analizador;
mod comandos;
mod ejecutor;
mod errores;
use comandos::comandos_implementacion::{parsear, Comandos};
use ejecutor::ejecutor_implementacion::ejecutar_comando;
use errores::errores_implementacion::{imprimir_error, Errores};
use std::env;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        imprimir_error(
            Errores::Error,
            "Los argumentos del programa fueron ingresados de manera incorrecta".to_string(),
        );
        return;
    }
    let path: &String = &args[1];
    let comando: &String = &args[2];
    let parseado: Comandos = match parsear(&comando.to_string()) {
        Ok(comando_parseado) => comando_parseado,
        _ => exit(1),
    };
    if let Ok(()) = ejecutar_comando(parseado, path) {}
}
