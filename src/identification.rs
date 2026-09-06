use crate::{IdSchema, Result, TileBatch, TileError, TileId, TileLocation, TileSurface};
use chrono::Utc;

fn encode_location(location: &TileLocation) -> String {
    let orbiter = if location.orbiter_id.len() >= 2 {
        location.orbiter_id[..2].to_string()
    } else {
        format!("{:0<2}", location.orbiter_id)
    };

    let surface = match &location.surface {
        TileSurface::NoseCap => "NC".to_string(),
        TileSurface::WingLeadingEdge => "WL".to_string(),
        TileSurface::FuselageTop => "FT".to_string(),
        TileSurface::FuselageBottom => "FB".to_string(),
        TileSurface::VerticalStabilizer => "VS".to_string(),
        TileSurface::OmsPod => "OM".to_string(),
        TileSurface::EjectionSeat => "ES".to_string(),
        TileSurface::Custom(s) => {
            if s.len() >= 2 {
                s[..2].to_string()
            } else {
                format!("{:0<2}", s)
            }
        }
    };

    let panel = if location.panel_id.len() >= 2 {
        location.panel_id[..2].to_string()
    } else {
        format!("{:0<2}", location.panel_id)
    };

    let row = if location.row <= 9 {
        (b'0' + location.row) as char
    } else {
        '9'
    };

    let col = if location.column <= 9 {
        (b'0' + location.column) as char
    } else {
        '9'
    };

    format!("{}{}{}{}{}", orbiter, surface, panel, row, col)
}

fn decode_location(code: &str) -> Result<TileLocation> {
    if code.len() != 8 {
        return Err(TileError::IdentificationError(
            "Location code must be 8 characters".to_string(),
        ));
    }

    let orbiter_id = format!("{}00", &code[..2]);
    let surface_code = &code[2..4];
    let panel_id = code[4..6].to_string();

    let row_b = code.as_bytes()[6];
    let col_b = code.as_bytes()[7];

    if !row_b.is_ascii_digit() || !col_b.is_ascii_digit() {
        return Err(TileError::IdentificationError(
            "Invalid row or column in location code".to_string(),
        ));
    }

    let row = row_b - b'0';
    let column = col_b - b'0';

    let surface = match surface_code {
        "NC" => TileSurface::NoseCap,
        "WL" => TileSurface::WingLeadingEdge,
        "FT" => TileSurface::FuselageTop,
        "FB" => TileSurface::FuselageBottom,
        "VS" => TileSurface::VerticalStabilizer,
        "OM" => TileSurface::OmsPod,
        "ES" => TileSurface::EjectionSeat,
        s => TileSurface::Custom(s.to_string()),
    };

    Ok(TileLocation {
        orbiter_id,
        surface,
        panel_id,
        row,
        column,
    })
}

fn extract_digits(s: &str) -> Vec<u8> {
    s.chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect()
}

fn luhn_checksum(digits: &[u8]) -> u8 {
    let mut sum = 0;
    let mut double = false;
    for &d in digits.iter().rev() {
        let mut value = d;
        if double {
            value *= 2;
            if value > 9 {
                value -= 9;
            }
        }
        sum += value;
        double = !double;
    }
    (100 - (sum % 100)) % 100
}

fn calculate_checksum_str(s: &str) -> u8 {
    let digits = extract_digits(s);
    luhn_checksum(&digits)
}

pub fn generate_id(batch: &TileBatch, location: &TileLocation, sequence: u32) -> Result<TileId> {
    if sequence > 9999 {
        return Err(TileError::IdentificationError(
            "Sequence must be 4 digits or less".to_string(),
        ));
    }

    let batch_code = if batch.batch_code.len() >= 6 {
        batch.batch_code[..6].to_string()
    } else {
        format!("{:0<6}", batch.batch_code)
    };

    let loc_code = encode_location(location);
    let seq_str = format!("{:04}", sequence);

    let prefix = format!("{}-{}-{}", batch_code, loc_code, seq_str);
    let checksum = calculate_checksum_str(&prefix);
    let checksum_str = format!("{:02}", checksum);

    let raw = format!("{}-{}", prefix, checksum_str);

    Ok(TileId {
        raw,
        batch: batch.clone(),
        location: location.clone(),
        sequence,
        checksum,
    })
}

pub fn parse_id(raw: &str, schema: &IdSchema) -> Result<TileId> {
    if !validate_schema(raw, schema)? {
        return Err(TileError::IdentificationError(
            "Schema validation failed".to_string(),
        ));
    }

    let parts: Vec<&str> = raw.split(schema.delimiter).collect();
    if parts.len() != 4 {
        return Err(TileError::IdentificationError(
            "Invalid ID format".to_string(),
        ));
    }

    let batch_code = parts[0].to_string();
    let loc_code = parts[1].to_string();
    let seq_str = parts[2];
    let checksum_str = parts[3];

    let sequence = seq_str.parse::<u32>().map_err(|_| {
        TileError::IdentificationError("Invalid sequence number".to_string())
    })?;

    let checksum = checksum_str.parse::<u8>().map_err(|_| {
        TileError::IdentificationError("Invalid checksum".to_string())
    })?;

    let prefix = format!(
        "{}{}{}{}{}",
        batch_code, schema.delimiter, loc_code, schema.delimiter, seq_str
    );
    let expected_checksum = calculate_checksum_str(&prefix);
    if checksum != expected_checksum {
        return Err(TileError::IdentificationError(
            "Checksum mismatch".to_string(),
        ));
    }

    let location = decode_location(&loc_code)?;
    let batch = TileBatch {
        batch_code: batch_code.clone(),
        production_date: Utc::now(),
        oven_id: String::new(),
        operator_id: String::new(),
    };

    Ok(TileId {
        raw: raw.to_string(),
        batch,
        location,
        sequence,
        checksum,
    })
}

pub fn encode_id(id: &TileId, schema: &IdSchema) -> String {
    let batch_code = if id.batch.batch_code.len() >= schema.batch_code_length {
        id.batch.batch_code[..schema.batch_code_length].to_string()
    } else {
        format!(
            "{:0<width$}",
            id.batch.batch_code,
            width = schema.batch_code_length
        )
    };

    let loc_code = encode_location(&id.location);
    let loc_code = if loc_code.len() >= schema.location_code_length {
        loc_code[..schema.location_code_length].to_string()
    } else {
        format!(
            "{:0<width$}",
            loc_code,
            width = schema.location_code_length
        )
    };

    let seq_str = format!("{:0width$}", id.sequence, width = schema.sequence_length);
    let prefix = format!(
        "{}{}{}{}{}",
        batch_code, schema.delimiter, loc_code, schema.delimiter, seq_str
    );
    let checksum = calculate_checksum_str(&prefix);
    let checksum_str = format!("{:0width$}", checksum, width = schema.checksum_length);

    format!("{}{}{}", prefix, schema.delimiter, checksum_str)
}

pub fn validate_checksum(id: &TileId) -> Result<bool> {
    let last_delim = id.raw.rfind('-').ok_or_else(|| {
        TileError::IdentificationError("No delimiter found".to_string())
    })?;

    let prefix = &id.raw[..last_delim];
    let expected = calculate_checksum_str(prefix);
    Ok(id.checksum == expected)
}

pub fn validate_schema(raw: &str, schema: &IdSchema) -> Result<bool> {
    let parts: Vec<&str> = raw.split(schema.delimiter).collect();
    if parts.len() != 4 {
        return Ok(false);
    }

    if parts[0].len() != schema.batch_code_length {
        return Ok(false);
    }

    if parts[1].len() != schema.location_code_length {
        return Ok(false);
    }

    if parts[2].len() != schema.sequence_length {
        return Ok(false);
    }

    if parts[3].len() != schema.checksum_length {
        return Ok(false);
    }

    if !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return Ok(false);
    }

    if !parts[3].chars().all(|c| c.is_ascii_digit()) {
        return Ok(false);
    }

    Ok(true)
}

pub fn to_human_readable(id: &TileId) -> String {
    format!(
        "Batch {}, Location {}-{}-{}-{}-{}, Sequence {:04}, Checksum {:02}",
        id.batch.batch_code,
        id.location.orbiter_id,
        id.location.surface,
        id.location.panel_id,
        id.location.row,
        id.location.column,
        id.sequence,
        id.checksum
    )
}

pub fn to_machine_readable(id: &TileId) -> String {
    id.raw.chars().filter(|c| c.is_alphanumeric()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_batch() -> TileBatch {
        TileBatch {
            batch_code: "A24001".to_string(),
            production_date: Utc::now(),
            oven_id: "Oven1".to_string(),
            operator_id: "OP001".to_string(),
        }
    }

    fn test_location() -> TileLocation {
        TileLocation {
            orbiter_id: "OV104".to_string(),
            surface: TileSurface::FuselageBottom,
            panel_id: "03".to_string(),
            row: 0,
            column: 3,
        }
    }

    fn default_schema() -> IdSchema {
        IdSchema::default()
    }

    #[test]
    fn test_generate_id_valid() {
        let batch = test_batch();
        let location = test_location();
        let id = generate_id(&batch, &location, 1).unwrap();

        assert_eq!(id.sequence, 1);
        assert_eq!(id.batch.batch_code, "A24001");
        assert_eq!(id.location.orbiter_id, "OV104");
        assert_eq!(id.location.surface, TileSurface::FuselageBottom);
        assert_eq!(id.location.panel_id, "03");
        assert_eq!(id.location.row, 0);
        assert_eq!(id.location.column, 3);
        assert!(id.raw.starts_with("A24001-OVFB0303-0001-"));
    }

    #[test]
    fn test_generate_id_sequence_max() {
        let batch = test_batch();
        let location = test_location();
        let id = generate_id(&batch, &location, 9999).unwrap();
        assert_eq!(id.sequence, 9999);
        assert!(id.raw.contains("-9999-"));
    }

    #[test]
    fn test_generate_id_sequence_overflow() {
        let batch = test_batch();
        let location = test_location();
        let result = generate_id(&batch, &location, 10000);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_id_short_batch() {
        let batch = TileBatch {
            batch_code: "ABC".to_string(),
            production_date: Utc::now(),
            oven_id: "O1".to_string(),
            operator_id: "OP".to_string(),
        };
        let location = test_location();
        let id = generate_id(&batch, &location, 1).unwrap();
        assert!(id.raw.starts_with("ABC000-"));
    }

    #[test]
    fn test_generate_id_long_batch() {
        let batch = TileBatch {
            batch_code: "ABCDEFGHI".to_string(),
            production_date: Utc::now(),
            oven_id: "O1".to_string(),
            operator_id: "OP".to_string(),
        };
        let location = test_location();
        let id = generate_id(&batch, &location, 1).unwrap();
        assert!(id.raw.starts_with("ABCDEF-"));
    }

    #[test]
    fn test_parse_id_valid() {
        let batch = test_batch();
        let location = test_location();
        let id = generate_id(&batch, &location, 1).unwrap();
        let schema = default_schema();
        let parsed = parse_id(&id.raw, &schema).unwrap();

        assert_eq!(parsed.sequence, id.sequence);
        assert_eq!(parsed.checksum, id.checksum);
        assert_eq!(parsed.raw, id.raw);
    }

    #[test]
    fn test_parse_id_invalid_checksum() {
        let raw = "A24001-OVFB0303-0001-00";
        let schema = default_schema();
        let result = parse_id(raw, &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_id_malformed_format() {
        let schema = default_schema();
        let result = parse_id("A24001-OVFB0303-0001", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_id_invalid_sequence() {
        let schema = default_schema();
        let result = parse_id("A24001-OVFB0303-ABCD-01", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_id_invalid_checksum_format() {
        let schema = default_schema();
        let result = parse_id("A24001-OVFB0303-0001-AB", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_id_wrong_batch_length() {
        let schema = default_schema();
        let result = parse_id("A2400-OVFB0303-0001-01", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_id_wrong_location_length() {
        let schema = default_schema();
        let result = parse_id("A24001-OVFB030-0001-01", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_id_default_schema() {
        let batch = test_batch();
        let location = test_location();
        let id = generate_id(&batch, &location, 42).unwrap();
        let schema = default_schema();
        let encoded = encode_id(&id, &schema);
        assert_eq!(encoded, id.raw);
    }

    #[test]
    fn test_encode_id_custom_schema() {
        let batch = test_batch();
        let location = test_location();
        let id = generate_id(&batch, &location, 42).unwrap();
        let schema = IdSchema {
            format: "BATCH-LOC-SEQ-CHECK".to_string(),
            batch_code_length: 6,
            location_code_length: 8,
            sequence_length: 4,
            checksum_length: 2,
            delimiter: ':',
        };
        let encoded = encode_id(&id, &schema);
        assert!(encoded.contains(':'));
        assert!(!encoded.contains('-'));
    }

    #[test]
    fn test_validate_checksum_valid() {
        let batch = test_batch();
        let location = test_location();
        let id = generate_id(&batch, &location, 1).unwrap();
        assert!(validate_checksum(&id).unwrap());
    }

    #[test]
    fn test_validate_checksum_invalid() {
        let mut id = generate_id(&test_batch(), &test_location(), 1).unwrap();
        id.checksum = 99;
        assert!(!validate_checksum(&id).unwrap());
    }

    #[test]
    fn test_validate_checksum_no_delimiter() {
        let id = TileId {
            raw: "INVALID".to_string(),
            batch: test_batch(),
            location: test_location(),
            sequence: 1,
            checksum: 0,
        };
        assert!(validate_checksum(&id).is_err());
    }

    #[test]
    fn test_validate_schema_valid() {
        let schema = default_schema();
        let id = generate_id(&test_batch(), &test_location(), 1).unwrap();
        assert!(validate_schema(&id.raw, &schema).unwrap());
    }

    #[test]
    fn test_validate_schema_wrong_part_count() {
        let schema = default_schema();
        assert!(!validate_schema("A-B-C", &schema).unwrap());
    }

    #[test]
    fn test_validate_schema_wrong_batch_length() {
        let schema = default_schema();
        assert!(!validate_schema("A2400-OVFB0303-0001-01", &schema).unwrap());
    }

    #[test]
    fn test_validate_schema_wrong_location_length() {
        let schema = default_schema();
        assert!(!validate_schema("A24001-OVFB030-0001-01", &schema).unwrap());
    }

    #[test]
    fn test_validate_schema_wrong_sequence_length() {
        let schema = default_schema();
        assert!(!validate_schema("A24001-OVFB0303-001-01", &schema).unwrap());
    }

    #[test]
    fn test_validate_schema_wrong_checksum_length() {
        let schema = default_schema();
        assert!(!validate_schema("A24001-OVFB0303-0001-1", &schema).unwrap());
    }

    #[test]
    fn test_validate_schema_non_digit_sequence() {
        let schema = default_schema();
        assert!(!validate_schema("A24001-OVFB0303-ABCD-01", &schema).unwrap());
    }

    #[test]
    fn test_validate_schema_non_digit_checksum() {
        let schema = default_schema();
        assert!(!validate_schema("A24001-OVFB0303-0001-AB", &schema).unwrap());
    }

    #[test]
    fn test_to_human_readable() {
        let batch = test_batch();
        let location = test_location();
        let id = generate_id(&batch, &location, 1).unwrap();
        let hr = to_human_readable(&id);
        assert!(hr.contains("Batch A24001"));
        assert!(hr.contains("Location"));
        assert!(hr.contains("Sequence 0001"));
        assert!(hr.contains("Checksum"));
    }

    #[test]
    fn test_to_machine_readable() {
        let batch = test_batch();
        let location = test_location();
        let id = generate_id(&batch, &location, 1).unwrap();
        let mr = to_machine_readable(&id);
        assert!(!mr.contains('-'));
        assert_eq!(mr.len(), id.raw.len() - 3);
    }

    #[test]
    fn test_custom_surface() {
        let location = TileLocation {
            orbiter_id: "OV104".to_string(),
            surface: TileSurface::Custom("XY".to_string()),
            panel_id: "03".to_string(),
            row: 0,
            column: 3,
        };
        let batch = test_batch();
        let id = generate_id(&batch, &location, 1).unwrap();
        assert!(id.raw.contains("XY"));

        let schema = default_schema();
        let parsed = parse_id(&id.raw, &schema).unwrap();
        assert_eq!(parsed.location.surface, TileSurface::Custom("XY".to_string()));
    }

    #[test]
    fn test_custom_surface_short() {
        let location = TileLocation {
            orbiter_id: "OV104".to_string(),
            surface: TileSurface::Custom("X".to_string()),
            panel_id: "03".to_string(),
            row: 0,
            column: 3,
        };
        let batch = test_batch();
        let id = generate_id(&batch, &location, 1).unwrap();
        assert!(id.raw.contains("X0"));
    }

    #[test]
    fn test_location_row_col_clamping() {
        let location = TileLocation {
            orbiter_id: "OV104".to_string(),
            surface: TileSurface::NoseCap,
            panel_id: "03".to_string(),
            row: 15,
            column: 20,
        };
        let encoded = encode_location(&location);
        assert_eq!(encoded.as_bytes()[6] as char, '9');
        assert_eq!(encoded.as_bytes()[7] as char, '9');
    }

    #[test]
    fn test_location_decode_invalid_length() {
        let result = decode_location("OVFB030");
        assert!(result.is_err());
    }

    #[test]
    fn test_location_decode_invalid_row_col() {
        let result = decode_location("OVFB03AB");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_surfaces() {
        let surfaces = vec![
            (TileSurface::NoseCap, "NC"),
            (TileSurface::WingLeadingEdge, "WL"),
            (TileSurface::FuselageTop, "FT"),
            (TileSurface::FuselageBottom, "FB"),
            (TileSurface::VerticalStabilizer, "VS"),
            (TileSurface::OmsPod, "OM"),
            (TileSurface::EjectionSeat, "ES"),
        ];

        for (surface, expected_code) in surfaces {
            let location = TileLocation {
                orbiter_id: "OV104".to_string(),
                surface,
                panel_id: "03".to_string(),
                row: 0,
                column: 3,
            };
            let encoded = encode_location(&location);
            assert!(encoded.contains(expected_code), "Expected {} in {}", expected_code, encoded);
        }
    }

    #[test]
    fn test_roundtrip_generate_parse() {
        let batch = test_batch();
        let location = test_location();
        let original = generate_id(&batch, &location, 123).unwrap();
        let schema = default_schema();
        let parsed = parse_id(&original.raw, &schema).unwrap();

        assert_eq!(original.sequence, parsed.sequence);
        assert_eq!(original.checksum, parsed.checksum);
        assert_eq!(original.raw, parsed.raw);
    }

    #[test]
    fn test_checksum_consistency() {
        let batch = test_batch();
        let location = test_location();
        let id1 = generate_id(&batch, &location, 1).unwrap();
        let id2 = generate_id(&batch, &location, 1).unwrap();
        assert_eq!(id1.checksum, id2.checksum);
        assert_eq!(id1.raw, id2.raw);
    }

    #[test]
    fn test_different_sequences_different_checksums() {
        let batch = test_batch();
        let location = test_location();
        let id1 = generate_id(&batch, &location, 1).unwrap();
        let id2 = generate_id(&batch, &location, 2).unwrap();
        assert_ne!(id1.checksum, id2.checksum);
    }
}
