//! Minimal reader for ENVI-met `.INX` area input files.
//!
//! The format is documented, as far as CLIMESH needs it, in `docs/formato-inx.md`.
//! The reader looks tags up by name over the whole text instead of walking a tree:
//! `.INX` is not well-formed XML (`<3Dplants>` is not a legal XML name) and every
//! tag CLIMESH reads is unique in the file, so the hierarchy carries no information
//! a strict parser would give back.

use std::fmt;
use std::path::Path;

/// The parts of an `.INX` file CLIMESH reads.
///
/// Every raster is optional: a section that is not in the file is `None`, which is
/// not the same thing as a section full of zeros or of empty cells.
#[derive(Debug, Clone)]
pub struct Inx {
    pub header: Header,
    pub geometry: Geometry,
    pub location: Location,
    /// Top height of the built volume, `<buildings2D>/<zTop>`.
    pub z_top: Option<Matrix<f64>>,
    /// Bottom height of the built volume, `<buildings2D>/<zBottom>`.
    pub z_bottom: Option<Matrix<f64>>,
    /// Building identifier per cell, `<buildings2D>/<buildingNr>`.
    pub building_nr: Option<Matrix<f64>>,
    /// Terrain height, `<dem>/<terrainheight>`.
    pub terrain_height: Option<Matrix<f64>>,
    /// Simple (2D) plant profile per cell, `<simpleplants2D>/<ID_plants1D>`.
    pub plants_2d: Option<Matrix<Option<String>>>,
    /// Soil profile per cell, `<soils2D>/<ID_soilprofile>`.
    pub soil_profiles: Option<Matrix<Option<String>>>,
    /// Emission source per cell, `<sources2D>/<ID_sources>`.
    pub sources: Option<Matrix<Option<String>>>,
    /// Every `<3Dplants>` instance, in file order.
    pub plants: Vec<Plant>,
}

/// What the file says about itself. Every field is optional: a file that does not
/// declare its format version is still readable, it just says less.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Header {
    pub filetype: Option<String>,
    pub version: Option<String>,
    pub revision_date: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub grids_i: usize,
    pub grids_j: usize,
    pub grids_z: usize,
    pub dx: f64,
    pub dy: f64,
    pub dz_base: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub name: String,
    pub longitude: f64,
    pub latitude: f64,
    /// Degrees the model is rotated by, clockwise from north.
    pub model_rotation: f64,
    pub timezone_name: String,
    pub timezone_longitude: f64,
}

/// One `<3Dplants>` instance: a single plant rooted in one cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Plant {
    /// ENVI-met cell index along x, 1-based.
    pub i: usize,
    /// ENVI-met cell index along y, 1-based, growing northwards.
    pub j: usize,
    /// ENVI-met cell index along z, 0-based.
    pub k: usize,
    pub plant_id: String,
    pub name: String,
    pub observe: bool,
}

/// A `matrix-data` block, stored in the order the rows are written.
///
/// The first row written is the northernmost one (`j == rows`); [`Matrix::at`]
/// takes ENVI-met indices and does the flip.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T> {
    /// `dataI`, the number of cells along x.
    pub cols: usize,
    /// `dataJ`, the number of cells along y.
    pub rows: usize,
    /// Cells in written order: first row first, west to east.
    pub cells: Vec<T>,
}

impl<T> Matrix<T> {
    /// The cell at ENVI-met indices `i` (1..=cols) and `j` (1..=rows), or `None`
    /// outside the grid.
    pub fn at(&self, i: usize, j: usize) -> Option<&T> {
        if i < 1 || i > self.cols || j < 1 || j > self.rows {
            return None;
        }
        self.cells.get((self.rows - j) * self.cols + (i - 1))
    }

    /// The rows in written order, northernmost first.
    pub fn rows_as_written(&self) -> impl Iterator<Item = &[T]> {
        self.cells.chunks(self.cols)
    }
}

#[derive(Debug)]
pub enum InxError {
    Io {
        path: String,
        source: std::io::Error,
    },
    NotAnInxFile,
    Encrypted {
        level: String,
    },
    MissingTag(String),
    UnclosedTag(String),
    MissingAttribute {
        tag: &'static str,
        attribute: &'static str,
    },
    NotANumber {
        tag: String,
        value: String,
    },
    MatrixRows {
        tag: &'static str,
        expected: usize,
        found: usize,
    },
    MatrixColumns {
        tag: &'static str,
        row: usize,
        expected: usize,
        found: usize,
    },
    CellNotANumber {
        tag: &'static str,
        row: usize,
        column: usize,
        j: usize,
        value: String,
    },
    PlantOutOfGrid {
        index: usize,
        field: &'static str,
        value: usize,
        limit_name: &'static str,
        limit: usize,
    },
}

impl fmt::Display for InxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InxError::Io { path, source } => write!(f, "cannot read INX file '{path}': {source}"),
            InxError::NotAnInxFile => {
                write!(f, "missing root tag <ENVI-MET_Datafile>: not an ENVI-met INX file")
            }
            InxError::Encrypted { level } => write!(
                f,
                "the file declares encryption level {level}: CLIMESH only reads plain text INX files"
            ),
            InxError::MissingTag(tag) => write!(f, "missing required tag <{tag}>"),
            InxError::UnclosedTag(tag) => write!(f, "missing closing tag </{tag}>"),
            InxError::MissingAttribute { tag, attribute } => {
                write!(f, "matrix <{tag}>: missing attribute {attribute}")
            }
            InxError::NotANumber { tag, value } => {
                write!(f, "tag <{tag}>: '{value}' is not a number")
            }
            InxError::MatrixRows { tag, expected, found } => write!(
                f,
                "matrix <{tag}>: dataJ declares {expected} rows, found {found}"
            ),
            InxError::MatrixColumns { tag, row, expected, found } => write!(
                f,
                "matrix <{tag}> row {row}: dataI declares {expected} columns, found {found}"
            ),
            InxError::CellNotANumber { tag, row, column, j, value } => write!(
                f,
                "matrix <{tag}> row {row}, column {column} (i={column}, j={j}): '{value}' is not a number"
            ),
            InxError::PlantOutOfGrid { index, field, value, limit_name, limit } => write!(
                f,
                "plant instance #{index}: {field} = {value} is outside the grid ({limit_name} = {limit})"
            ),
        }
    }
}

impl std::error::Error for InxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InxError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Reads an `.INX` file from disk. Bytes that are not valid UTF-8 are replaced
/// rather than refused: the files come from a program CLIMESH does not control.
pub fn read_inx(path: impl AsRef<Path>) -> Result<Inx, InxError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|source| InxError::Io {
        path: path.display().to_string(),
        source,
    })?;
    parse_inx(&String::from_utf8_lossy(&bytes))
}

pub fn parse_inx(text: &str) -> Result<Inx, InxError> {
    if find_open(text, "ENVI-MET_Datafile", 0).is_none() {
        return Err(InxError::NotAnInxFile);
    }
    if let Some(level) = scalar(text, "encryptionlevel")? {
        if level != "0" {
            return Err(InxError::Encrypted {
                level: level.to_string(),
            });
        }
    }
    let header = Header {
        filetype: scalar(text, "filetype")?.map(str::to_string),
        version: scalar(text, "version")?.map(str::to_string),
        revision_date: scalar(text, "revisiondate")?.map(str::to_string),
        remark: scalar(text, "remark")?.map(str::to_string),
    };
    let geometry = Geometry {
        grids_i: required_usize(text, "grids-I")?,
        grids_j: required_usize(text, "grids-J")?,
        grids_z: required_usize(text, "grids-Z")?,
        dx: required_f64(text, "dx")?,
        dy: required_f64(text, "dy")?,
        dz_base: required_f64(text, "dz-base")?,
    };
    let location = Location {
        name: required_str(text, "locationName")?,
        longitude: required_f64(text, "location_Longitude")?,
        latitude: required_f64(text, "location_Latitude")?,
        model_rotation: required_f64(text, "modelRotation")?,
        timezone_name: required_str(text, "locationTimeZone_Name")?,
        timezone_longitude: required_f64(text, "locationTimeZone_Longitude")?,
    };
    Ok(Inx {
        header,
        z_top: numeric_matrix(text, "zTop")?,
        z_bottom: numeric_matrix(text, "zBottom")?,
        building_nr: numeric_matrix(text, "buildingNr")?,
        terrain_height: numeric_matrix(text, "terrainheight")?,
        plants_2d: id_matrix(text, "ID_plants1D")?,
        soil_profiles: id_matrix(text, "ID_soilprofile")?,
        sources: id_matrix(text, "ID_sources")?,
        plants: plants(text, &geometry)?,
        geometry,
        location,
    })
}

/// Start of the opening tag `tag` at or after `from`, if any.
fn find_open(text: &str, tag: &str, from: usize) -> Option<usize> {
    let pattern = format!("<{tag}");
    let mut cursor = from;
    while let Some(offset) = text[cursor..].find(&pattern) {
        let start = cursor + offset;
        let after = start + pattern.len();
        match text[after..].chars().next() {
            Some('>') | Some(' ') | Some('\t') | Some('\r') | Some('\n') => return Some(start),
            _ => cursor = after,
        }
    }
    None
}

/// The first element named `tag` at or after `from`: its attributes, its content,
/// and the offset just past its closing tag.
fn element<'a>(
    text: &'a str,
    tag: &str,
    from: usize,
) -> Result<Option<(&'a str, &'a str, usize)>, InxError> {
    let Some(start) = find_open(text, tag, from) else {
        return Ok(None);
    };
    let unclosed = || InxError::UnclosedTag(tag.to_string());
    let head_end = start + text[start..].find('>').ok_or_else(unclosed)?;
    let closing = format!("</{tag}>");
    let close_at = head_end + text[head_end..].find(&closing).ok_or_else(unclosed)?;
    Ok(Some((
        &text[start + 1 + tag.len()..head_end],
        &text[head_end + 1..close_at],
        close_at + closing.len(),
    )))
}

fn scalar<'a>(text: &'a str, tag: &str) -> Result<Option<&'a str>, InxError> {
    Ok(element(text, tag, 0)?.map(|(_, content, _)| content.trim()))
}

fn required_str(text: &str, tag: &str) -> Result<String, InxError> {
    scalar(text, tag)?
        .map(str::to_string)
        .ok_or_else(|| InxError::MissingTag(tag.to_string()))
}

fn required_f64(text: &str, tag: &str) -> Result<f64, InxError> {
    let raw = required_str(text, tag)?;
    raw.parse().map_err(|_| InxError::NotANumber {
        tag: tag.to_string(),
        value: raw,
    })
}

fn required_usize(text: &str, tag: &str) -> Result<usize, InxError> {
    let raw = required_str(text, tag)?;
    raw.parse().map_err(|_| InxError::NotANumber {
        tag: tag.to_string(),
        value: raw,
    })
}

fn attribute<'a>(attributes: &'a str, name: &str) -> Option<&'a str> {
    let pattern = format!("{name}=\"");
    let start = attributes.find(&pattern)? + pattern.len();
    let end = start + attributes[start..].find('"')?;
    Some(&attributes[start..end])
}

/// A `matrix-data` block with its cells still as written.
fn raw_matrix<'a>(text: &'a str, tag: &'static str) -> Result<Option<Matrix<&'a str>>, InxError> {
    let Some((attributes, content, _)) = element(text, tag, 0)? else {
        return Ok(None);
    };
    let size = |name: &'static str| -> Result<usize, InxError> {
        let raw = attribute(attributes, name).ok_or(InxError::MissingAttribute {
            tag,
            attribute: name,
        })?;
        raw.trim().parse().map_err(|_| InxError::NotANumber {
            tag: format!("{tag} {name}"),
            value: raw.to_string(),
        })
    };
    let cols = size("dataI")?;
    let rows = size("dataJ")?;

    let mut cells = Vec::with_capacity(cols * rows);
    let mut written_rows = 0;
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        written_rows += 1;
        let row: Vec<&str> = line.trim().split(',').map(str::trim).collect();
        if row.len() != cols {
            return Err(InxError::MatrixColumns {
                tag,
                row: written_rows,
                expected: cols,
                found: row.len(),
            });
        }
        cells.extend(row);
    }
    if written_rows != rows {
        return Err(InxError::MatrixRows {
            tag,
            expected: rows,
            found: written_rows,
        });
    }
    Ok(Some(Matrix { cols, rows, cells }))
}

fn numeric_matrix(text: &str, tag: &'static str) -> Result<Option<Matrix<f64>>, InxError> {
    let Some(raw) = raw_matrix(text, tag)? else {
        return Ok(None);
    };
    let mut cells = Vec::with_capacity(raw.cells.len());
    for (index, value) in raw.cells.iter().enumerate() {
        let row = index / raw.cols + 1;
        cells.push(value.parse::<f64>().map_err(|_| InxError::CellNotANumber {
            tag,
            row,
            column: index % raw.cols + 1,
            j: raw.rows - row + 1,
            value: (*value).to_string(),
        })?);
    }
    Ok(Some(Matrix {
        cols: raw.cols,
        rows: raw.rows,
        cells,
    }))
}

/// An identifier matrix, where an empty cell means "nothing here" and not "zero".
fn id_matrix(text: &str, tag: &'static str) -> Result<Option<Matrix<Option<String>>>, InxError> {
    Ok(raw_matrix(text, tag)?.map(|raw| Matrix {
        cols: raw.cols,
        rows: raw.rows,
        cells: raw
            .cells
            .iter()
            .map(|value| {
                if value.is_empty() {
                    None
                } else {
                    Some((*value).to_string())
                }
            })
            .collect(),
    }))
}

fn plants(text: &str, geometry: &Geometry) -> Result<Vec<Plant>, InxError> {
    let mut plants = Vec::new();
    let mut cursor = 0;
    while let Some((_, block, next)) = element(text, "3Dplants", cursor)? {
        cursor = next;
        let index = plants.len() + 1;
        let field = |tag: &str| -> Result<String, InxError> {
            scalar(block, tag)?
                .map(str::to_string)
                .ok_or_else(|| InxError::MissingTag(format!("{tag} in 3Dplants #{index}")))
        };
        let cell = |tag: &'static str| -> Result<usize, InxError> {
            let raw = field(tag)?;
            raw.parse().map_err(|_| InxError::NotANumber {
                tag: format!("{tag} in 3Dplants #{index}"),
                value: raw,
            })
        };
        let plant = Plant {
            i: cell("rootcell_i")?,
            j: cell("rootcell_j")?,
            k: cell("rootcell_k")?,
            plant_id: field("plantID")?,
            name: field("name")?,
            observe: field("observe")? != "0",
        };
        // i and j are 1-based, k is 0-based: see docs/formato-inx.md.
        let checks = [
            (
                "rootcell_i",
                plant.i,
                plant.i < 1 || plant.i > geometry.grids_i,
                "grids-I",
                geometry.grids_i,
            ),
            (
                "rootcell_j",
                plant.j,
                plant.j < 1 || plant.j > geometry.grids_j,
                "grids-J",
                geometry.grids_j,
            ),
            (
                "rootcell_k",
                plant.k,
                plant.k >= geometry.grids_z,
                "grids-Z",
                geometry.grids_z,
            ),
        ];
        for (field, value, out_of_grid, limit_name, limit) in checks {
            if out_of_grid {
                return Err(InxError::PlantOutOfGrid {
                    index,
                    field,
                    value,
                    limit_name,
                    limit,
                });
            }
        }
        plants.push(plant);
    }
    Ok(plants)
}
