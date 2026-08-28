#[derive(Clone, Copy)]
pub enum BytesUnit {
    B,
    KB,
    MB,
    GB,
    TB,
}

const KB_SIZE: usize = 1024;
const MB_SIZE: usize = KB_SIZE * 1024;
const GB_SIZE: usize = MB_SIZE * 1024;
const TB_SIZE: usize = GB_SIZE * 1024;

impl BytesUnit {
    pub fn to_bytes(&self) -> usize {
        match self {
            Self::B => 1,
            Self::KB => KB_SIZE,
            Self::MB => MB_SIZE,
            Self::GB => GB_SIZE,
            Self::TB => TB_SIZE,
        }
    }
}

impl std::fmt::Display for BytesUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::B => f.write_str("B"),
            Self::KB => f.write_str("KB"),
            Self::MB => f.write_str("MB"),
            Self::GB => f.write_str("GB"),
            Self::TB => f.write_str("TB"),
        }
    }
}

pub fn best_unit_for_bytes(bytes: usize) -> BytesUnit {
    match bytes {
        0..KB_SIZE => BytesUnit::B,
        KB_SIZE..MB_SIZE =>BytesUnit::KB,
        MB_SIZE..GB_SIZE => BytesUnit::MB,
        GB_SIZE..TB_SIZE => BytesUnit::GB,
        _ => BytesUnit::TB,
    }
}

pub fn convert_bytes_to_unit(bytes: usize, unit: BytesUnit) -> f32 {
    bytes as f32/ unit.to_bytes() as f32
}

pub fn bytes_to_string(bytes: usize, precision: usize) -> String {
    let unit = best_unit_for_bytes(bytes);
    match unit {
        BytesUnit::B => format!("{bytes}{unit}"),
        _ => {
            let value = convert_bytes_to_unit(bytes, unit);
            format!("{value:.prec$}{unit}", prec = precision)
        }
    }
}
