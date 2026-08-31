//! Behaviour of the minimal ENVI-met `.INX` reader, degenerate inputs first.

use climesh::inx::{parse_inx, read_inx};

/// Wraps a body in the smallest header a valid `.INX` file has: a 3x2x4 grid,
/// deliberately not the 50x50x25 of the reference case.
fn wrap(body: &str) -> String {
    format!(
        "<ENVI-MET_Datafile>
<modelGeometry>
   <grids-I> 3 </grids-I>
   <grids-J> 2 </grids-J>
   <grids-Z> 4 </grids-Z>
   <dx> 1.50000 </dx>
   <dy> 1.50000 </dy>
   <dz-base> 2.00000 </dz-base>
</modelGeometry>
<locationData>
   <modelRotation> 21.00000 </modelRotation>
   <locationName> bastia </locationName>
   <location_Longitude> 12.56000 </location_Longitude>
   <location_Latitude> 43.07000 </location_Latitude>
   <locationTimeZone_Name> CET/ UTC+1 </locationTimeZone_Name>
   <locationTimeZone_Longitude> 15.00000 </locationTimeZone_Longitude>
</locationData>
{body}
</ENVI-MET_Datafile>
"
    )
}

fn z_top(rows: &str) -> String {
    wrap(&format!(
        "<buildings2D>
   <zTop type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
{rows}
   </zTop>
</buildings2D>"
    ))
}

#[test]
fn missing_file_error_names_the_path() {
    let err = read_inx("/percorso/che/non/esiste/LAB1.INX").unwrap_err();
    assert!(
        err.to_string()
            .contains("/percorso/che/non/esiste/LAB1.INX"),
        "error should name the path, was: {err}"
    );
}

#[test]
fn a_file_without_the_root_tag_is_rejected() {
    let err = parse_inx("<html><body>not an INX at all</body></html>").unwrap_err();
    assert!(
        err.to_string().contains("ENVI-MET_Datafile"),
        "error should name the expected root tag, was: {err}"
    );
}

#[test]
fn a_matrix_without_its_closing_tag_names_the_tag() {
    let text = wrap(
        "<buildings2D>
   <zTop type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
   0,0,0
   0,0,0",
    );
    let err = parse_inx(&text).unwrap_err();
    assert!(
        err.to_string().contains("zTop"),
        "error should name the unclosed tag, was: {err}"
    );
}

#[test]
fn a_scalar_without_its_closing_tag_names_the_tag() {
    let err = parse_inx(
        "<ENVI-MET_Datafile>
<modelGeometry>
   <grids-I> 3
</modelGeometry>
</ENVI-MET_Datafile>",
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("grids-I"),
        "error should name the unclosed tag, was: {err}"
    );
}

#[test]
fn a_matrix_with_too_few_rows_reports_expected_and_found() {
    let err = parse_inx(&z_top("   0,0,0")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("zTop"), "was: {msg}");
    assert!(
        msg.contains('2') && msg.contains('1'),
        "message should report 2 expected and 1 found, was: {msg}"
    );
}

#[test]
fn a_matrix_row_with_the_wrong_width_reports_row_expected_and_found() {
    let err = parse_inx(&z_top("   0,0,0\n   0,0,0,0")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("zTop"), "was: {msg}");
    assert!(
        msg.contains("row 2"),
        "message should name the offending row, was: {msg}"
    );
    assert!(
        msg.contains('3') && msg.contains('4'),
        "message should report 3 expected and 4 found, was: {msg}"
    );
}

#[test]
fn numeric_cells_are_read_through_the_spaces_around_them() {
    let inx = parse_inx(&z_top("   0 , 6.5,0\n  0,0 ,  3 ")).unwrap();
    let z = inx.z_top.expect("zTop should be present");
    assert_eq!(z.at(2, 2), Some(&6.5));
    assert_eq!(z.at(3, 1), Some(&3.0));
}

#[test]
fn empty_cells_of_an_id_matrix_are_read_as_absent() {
    let text = wrap(
        "<simpleplants2D>
   <ID_plants1D type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
   0100XX, ,0100XX
   ,0100XX,
   </ID_plants1D>
</simpleplants2D>",
    );
    let m = parse_inx(&text)
        .unwrap()
        .plants_2d
        .expect("ID_plants1D should be present");
    assert_eq!(m.at(1, 2), Some(&Some("0100XX".to_string())));
    assert_eq!(m.at(2, 2), Some(&None));
    assert_eq!(m.at(1, 1), Some(&None));
    assert_eq!(m.at(2, 1), Some(&Some("0100XX".to_string())));
}

#[test]
fn a_cell_that_is_not_a_number_reports_its_row_and_column() {
    let err = parse_inx(&z_top("   0,0,0\n   0,muro,0")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("zTop"), "was: {msg}");
    assert!(
        msg.contains("row 2"),
        "message should name the row, was: {msg}"
    );
    assert!(
        msg.contains("column 2"),
        "message should name the column, was: {msg}"
    );
    assert!(
        msg.contains("muro"),
        "message should quote the offending value, was: {msg}"
    );
}

#[test]
fn an_absent_section_is_absent_and_not_a_matrix_of_zeros() {
    let inx = parse_inx(&z_top("   0,0,0\n   0,0,0")).unwrap();
    assert!(
        inx.sources.is_none(),
        "a file without <sources2D> must report absence"
    );
}

#[test]
fn a_section_present_but_wholly_empty_is_distinguishable_from_an_absent_one() {
    let text = wrap(
        "<sources2D>
   <ID_sources type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
   ,,
   ,,
   </ID_sources>
</sources2D>",
    );
    let m = parse_inx(&text)
        .unwrap()
        .sources
        .expect("<sources2D> is present, even if empty");
    assert_eq!(m.cells.len(), 6);
    assert!(m.cells.iter().all(|c| c.is_none()));
}

#[test]
fn the_grid_size_comes_from_the_file_and_is_not_wired_in() {
    let inx = parse_inx(&z_top("   1,2,3\n   4,5,6")).unwrap();
    assert_eq!(
        (
            inx.geometry.grids_i,
            inx.geometry.grids_j,
            inx.geometry.grids_z
        ),
        (3, 2, 4)
    );
    assert_eq!(
        (inx.geometry.dx, inx.geometry.dy, inx.geometry.dz_base),
        (1.5, 1.5, 2.0)
    );
    assert_eq!(inx.location.latitude, 43.07);
    assert_eq!(inx.location.name, "bastia");
}

#[test]
fn the_first_row_written_is_the_northernmost_one() {
    let z = parse_inx(&z_top("   1,2,3\n   4,5,6"))
        .unwrap()
        .z_top
        .unwrap();
    assert_eq!(
        z.at(1, 2),
        Some(&1.0),
        "i=1, j=grids-J is the first value written"
    );
    assert_eq!(z.at(3, 2), Some(&3.0));
    assert_eq!(z.at(1, 1), Some(&4.0), "j=1 is the last row written");
    assert_eq!(z.at(3, 1), Some(&6.0));
    assert_eq!(z.at(4, 1), None, "outside the grid there is no cell");
    assert_eq!(z.at(0, 1), None, "ENVI-met indices start at 1");
}

#[test]
fn plant_instances_carry_position_species_and_observation_flag() {
    let text = wrap(
        "<3Dplants>
   <rootcell_i> 2 </rootcell_i>
   <rootcell_j> 1 </rootcell_j>
   <rootcell_k> 0 </rootcell_k>
   <plantID> 020027 </plantID>
   <name> .Pine Tree (middle) </name>
   <observe> 1 </observe>
</3Dplants>",
    );
    let plants = parse_inx(&text).unwrap().plants;
    assert_eq!(plants.len(), 1);
    assert_eq!((plants[0].i, plants[0].j, plants[0].k), (2, 1, 0));
    assert_eq!(plants[0].plant_id, "020027");
    assert_eq!(plants[0].name, ".Pine Tree (middle)");
    assert!(plants[0].observe);
}

#[test]
fn a_plant_rooted_outside_the_grid_is_rejected() {
    let above_the_grid = wrap(
        "<3Dplants>
   <rootcell_i> 2 </rootcell_i>
   <rootcell_j> 1 </rootcell_j>
   <rootcell_k> 9 </rootcell_k>
   <plantID> 020027 </plantID>
   <name> .Pine Tree (middle) </name>
   <observe> 0 </observe>
</3Dplants>",
    );
    let msg = parse_inx(&above_the_grid).unwrap_err().to_string();
    assert!(
        msg.contains("rootcell_k"),
        "error should name the field, was: {msg}"
    );
    assert!(
        msg.contains('9') && msg.contains('4'),
        "error should report value and limit, was: {msg}"
    );

    let beside_the_grid = wrap(
        "<3Dplants>
   <rootcell_i> 7 </rootcell_i>
   <rootcell_j> 1 </rootcell_j>
   <rootcell_k> 0 </rootcell_k>
   <plantID> 020027 </plantID>
   <name> .Pine Tree (middle) </name>
   <observe> 0 </observe>
</3Dplants>",
    );
    let msg = parse_inx(&beside_the_grid).unwrap_err().to_string();
    assert!(
        msg.contains("rootcell_i"),
        "error should name the field, was: {msg}"
    );
}

/// The reference case itself. `materiale università/` is not redistributable and
/// stays out of the repository, so this check only runs where the file is present.
#[test]
fn the_reference_case_is_read_whole() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/materiale università/LAB1.INX");
    if !std::path::Path::new(path).exists() {
        eprintln!("LAB1.INX assente: verifica del caso di riferimento saltata");
        return;
    }
    let inx = read_inx(path).unwrap();
    assert_eq!(
        (
            inx.geometry.grids_i,
            inx.geometry.grids_j,
            inx.geometry.grids_z
        ),
        (50, 50, 25)
    );
    assert_eq!(inx.location.name, "bergamo");
    assert_eq!(inx.plants.len(), 616);

    let z = inx.z_top.as_ref().unwrap();
    assert_eq!(
        z.at(11, 26),
        Some(&6.0),
        "corner of the north-west duplex block"
    );
    assert_eq!(z.at(23, 18), Some(&3.0), "corner of the simplex block");
    assert_eq!(
        z.at(15, 18),
        Some(&0.0),
        "observation point 1 is on open ground"
    );

    let sources = inx
        .sources
        .as_ref()
        .expect("<sources2D> is present in LAB1.INX");
    assert!(
        sources.cells.iter().all(|c| c.is_none()),
        "no source is placed"
    );
    let soils = inx.soil_profiles.as_ref().unwrap();
    assert!(
        soils.cells.iter().all(|c| c.as_deref() == Some("000000")),
        "one single soil profile"
    );
}

#[test]
fn the_header_says_which_program_and_format_version_wrote_the_file() {
    let text = wrap(
        "<Header>
   <filetype>INPX ENVI-met Area Input File</filetype>
   <version>440</version>
   <revisiondate>15/03/2023 17:32:25</revisiondate>
   <remark>Created with SPACES 5.1.1</remark>
   <checksum>0</checksum>
   <encryptionlevel>0</encryptionlevel>
</Header>",
    );
    let header = parse_inx(&text).unwrap().header;
    assert_eq!(
        header.filetype.as_deref(),
        Some("INPX ENVI-met Area Input File")
    );
    assert_eq!(header.version.as_deref(), Some("440"));
    assert_eq!(header.revision_date.as_deref(), Some("15/03/2023 17:32:25"));
    assert_eq!(header.remark.as_deref(), Some("Created with SPACES 5.1.1"));
}

#[test]
fn an_encrypted_file_is_refused_instead_of_being_read_as_garbage() {
    let text = wrap("<Header>\n   <encryptionlevel>1</encryptionlevel>\n</Header>");
    let msg = parse_inx(&text).unwrap_err().to_string();
    assert!(
        msg.contains("encryption") && msg.contains('1'),
        "error should say the file is encrypted and at which level, was: {msg}"
    );
}
