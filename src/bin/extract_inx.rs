//! Dumps an ENVI-met `.INX` file as TOML, so that the case can live in the
//! repository without the original, which is not redistributable.
//!
//! Usage: `cargo run --bin extract_inx -- "materiale università/LAB1.INX" > casi/bastia/scenario-lab1.toml`

use std::fmt::Write as _;

use climesh::inx::{read_inx, Matrix};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("uso: extract_inx <file.INX>");
        std::process::exit(2);
    };
    let inx = read_inx(&path)?;
    let source = std::path::Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.clone());

    let mut out = String::new();
    writeln!(
        out,
        "# Estratto di un file ENVI-met .INX, generato da `cargo run --bin extract_inx`."
    )?;
    writeln!(
        out,
        "# File generato: non modificarlo a mano. La struttura del formato di partenza"
    )?;
    writeln!(out, "# è descritta in docs/formato-inx.md.\n")?;

    writeln!(out, "[sorgente]")?;
    writeln!(out, "file = {}", quote(&source))?;
    writeln!(out, "byte = {}", std::fs::metadata(&path)?.len())?;
    for (key, value) in [
        ("filetype", &inx.header.filetype),
        ("versione_formato", &inx.header.version),
        ("data_revisione", &inx.header.revision_date),
        ("nota", &inx.header.remark),
    ] {
        if let Some(value) = value {
            writeln!(out, "{key} = {}", quote(value))?;
        }
    }

    let g = &inx.geometry;
    writeln!(out, "\n[griglia]")?;
    writeln!(
        out,
        "grids_i = {}\ngrids_j = {}\ngrids_z = {}",
        g.grids_i, g.grids_j, g.grids_z
    )?;
    writeln!(
        out,
        "dx = {:?}\ndy = {:?}\ndz_base = {:?}",
        g.dx, g.dy, g.dz_base
    )?;

    let l = &inx.location;
    writeln!(out, "\n[localizzazione]")?;
    writeln!(out, "nome = {}", quote(&l.name))?;
    writeln!(
        out,
        "longitudine = {:?}\nlatitudine = {:?}",
        l.longitude, l.latitude
    )?;
    writeln!(out, "rotazione_modello = {:?}", l.model_rotation)?;
    writeln!(out, "fuso_orario = {}", quote(&l.timezone_name))?;
    writeln!(out, "fuso_orario_longitudine = {:?}", l.timezone_longitude)?;

    writeln!(
        out,
        "\n# Le matrici sono scritte come nel file .INX: la prima riga è quella più a nord,"
    )?;
    writeln!(
        out,
        "# cioè j = grids_j, l'ultima è j = 1; dentro la riga i cresce da ovest a est."
    )?;
    writeln!(
        out,
        "# Una sezione assente dal file .INX è assente anche qui; nelle matrici di"
    )?;
    writeln!(
        out,
        "# identificativi la stringa vuota è una cella senza nulla dentro."
    )?;
    writeln!(out, "\n[matrici]")?;
    for (key, matrix) in [
        ("z_top", &inx.z_top),
        ("z_bottom", &inx.z_bottom),
        ("building_nr", &inx.building_nr),
        ("terrain_height", &inx.terrain_height),
    ] {
        if let Some(matrix) = matrix {
            write_matrix(&mut out, key, matrix, |value| format!("{value:?}"))?;
        }
    }
    for (key, matrix) in [
        ("id_plants_1d", &inx.plants_2d),
        ("id_soilprofile", &inx.soil_profiles),
        ("id_sources", &inx.sources),
    ] {
        if let Some(matrix) = matrix {
            write_matrix(&mut out, key, matrix, |value| {
                quote(value.as_deref().unwrap_or(""))
            })?;
        }
    }

    writeln!(
        out,
        "\n# Un'istanza per riga, nell'ordine in cui stanno nel file .INX."
    )?;
    writeln!(out, "# i e j sono indici di cella 1-based, k è 0-based.")?;
    writeln!(out, "[vegetazione]")?;
    writeln!(out, "alberi = [")?;
    for plant in &inx.plants {
        writeln!(
            out,
            "  {{ i = {}, j = {}, k = {}, plant_id = {}, nome = {}, osservato = {} }},",
            plant.i,
            plant.j,
            plant.k,
            quote(&plant.plant_id),
            quote(&plant.name),
            plant.observe
        )?;
    }
    writeln!(out, "]")?;

    print!("{out}");
    Ok(())
}

fn write_matrix<T>(
    out: &mut String,
    key: &str,
    matrix: &Matrix<T>,
    cell: impl Fn(&T) -> String,
) -> std::fmt::Result {
    writeln!(out, "{key} = [")?;
    for row in matrix.rows_as_written() {
        let row: Vec<String> = row.iter().map(&cell).collect();
        writeln!(out, "  [{}],", row.join(","))?;
    }
    writeln!(out, "]")
}

fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
