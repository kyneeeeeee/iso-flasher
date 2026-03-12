use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

fn main() {
    println!("╔══════════════════════════════╗");
    println!("║     iso-flasher v1.0         ║");
    println!("╚══════════════════════════════╝");
    println!("Escribe 'ayuda' para ver los comandos disponibles.\n");

    let stdin = io::stdin();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.lock().read_line(&mut input).unwrap();
        let input = input.trim();


        let partes: Vec<&str> = input.splitn(3, ' ').collect();
        let comando = partes[0];

        match comando {
            "ayuda" | "help" => {
                println!("Comandos disponibles:");
                println!("  flashear <ruta_iso> <dispositivo>   Flashea un ISO en un USB");
                println!("  dispositivos                         Lista los dispositivos disponibles");
                println!("  salir                                Cierra el programa");
            }

            "flashear" => {
                if partes.len() < 3 {
                    println!("Uso: flashear <ruta_iso> <dispositivo>");
                    println!("Ejemplo: flashear ubuntu.iso /dev/sdb");
                    continue;
                }
                let iso_path = partes[1];
                let usb_device = partes[2];
                flashear(iso_path, usb_device);
            }

            "dispositivos" => {
                listar_dispositivos();
            }

            "salir" | "exit" | "q" => {
                println!("¡Hasta luego!");
                break;
            }

            "" => {
                
            }

            _ => {
                println!("Comando desconocido: '{}'. Escribe 'ayuda' para ver los comandos.", comando);
            }
        }
    }
}

fn flashear(iso_path: &str, usb_device: &str) {
    if !Path::new(iso_path).exists() {
        println!("Error: No se encontró el archivo ISO: {}", iso_path);
        return;
    }

    let iso_size = match std::fs::metadata(iso_path) {
        Ok(m) => m.len(),
        Err(e) => { println!("Error al leer el ISO: {}", e); return; }
    };
    let iso_size_mb = iso_size / 1_048_576;

    println!("ISO:         {}", iso_path);
    println!("Tamaño:      {} MB", iso_size_mb);
    println!("Dispositivo: {}", usb_device);
    println!();
    println!("¡ADVERTENCIA! Esto borrará TODO el contenido de {}", usb_device);
    print!("¿Continuar? (escribe 'si' para confirmar): ");
    io::stdout().flush().unwrap();

    let mut confirmacion = String::new();
    io::stdin().lock().read_line(&mut confirmacion).unwrap();
    if confirmacion.trim() != "si" {
        println!("Operación cancelada.\n");
        return;
    }

    let iso_file = match File::open(iso_path) {
        Ok(f) => f,
        Err(e) => { println!("Error al abrir el ISO: {}", e); return; }
    };
    let mut reader = BufReader::with_capacity(4 * 1024 * 1024, iso_file);

    let mut usb = match OpenOptions::new().write(true).open(usb_device) {
        Ok(f) => f,
        Err(e) => {
            println!("Error al abrir {}: {}", usb_device, e);
            println!("¿Estás ejecutando con sudo/permisos de root?");
            return;
        }
    };

    println!("\nFlasheando...");
    let mut buffer = vec![0u8; 4 * 1024 * 1024];
    let mut bytes_escritos: u64 = 0;

    loop {
        let n = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => { println!("\nError durante la escritura: {}", e); return; }
        };
        if n == 0 { break; }

        if let Err(e) = usb.write_all(&buffer[..n]) {
            println!("\nError escribiendo en el USB: {}", e);
            return;
        }

        bytes_escritos += n as u64;
        let porcentaje = (bytes_escritos * 100) / iso_size;
        let mb_escritos = bytes_escritos / 1_048_576;
        print!("\r  Progreso: {}% ({}/{} MB)", porcentaje, mb_escritos, iso_size_mb);
        io::stdout().flush().unwrap();
    }

    usb.flush().unwrap();
    println!("\n\n¡Listo! USB flasheado correctamente.\n");
}

fn listar_dispositivos() {
    println!("Dispositivos de bloque disponibles:");

    #[cfg(target_os = "linux")]
    {
        match std::process::Command::new("lsblk").args(["-o", "NAME,SIZE,TYPE,MOUNTPOINT"]).output() {
            Ok(output) => println!("{}", String::from_utf8_lossy(&output.stdout)),
            Err(_) => println!("  No se pudo ejecutar lsblk. Revisa manualmente con: lsblk"),
        }
    }

    #[cfg(target_os = "macos")]
    {
        match std::process::Command::new("diskutil").arg("list").output() {
            Ok(output) => println!("{}", String::from_utf8_lossy(&output.stdout)),
            Err(_) => println!("  No se pudo ejecutar diskutil."),
        }
    }

    #[cfg(target_os = "windows")]
    {
        println!("  En Windows usa: wmic diskdrive list brief");
    }
}
