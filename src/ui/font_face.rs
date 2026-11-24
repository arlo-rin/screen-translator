//! Picking a named face out of a `.ttf` / `.ttc` file.
//!
//! Windows and macOS ship CJK fonts as collections: Microsoft YaHei and
//! Microsoft YaHei UI live in the same `msyh.ttc`, Yu Gothic and Yu Gothic UI
//! in `YuGoth*.ttc`, and so on. The first face is usually the document face,
//! whose typographic metrics (and, for Yu Gothic, a huge line gap) do not
//! match the Latin UI font. Asking egui for index 0 therefore mixed Segoe UI
//! Latin with a Japanese body face — and a downward `y_offset_factor` on that
//! body face made mixed labels look as if the characters were jumping up and
//! down.
//!
//! We look up the face by typographic name instead, and skip the candidate
//! when that name is not in the file, so a `.ttc` whose second face happens
//! to be italic is never taken by accident.

/// Index of the face named `preferred` in `bytes`, if it is there.
///
/// `preferred` may be a family ("Microsoft YaHei UI") or a full name
/// ("Yu Gothic UI Regular"). A family match only accepts a Regular (or
/// unnamed) subfamily, so "Yu Gothic UI" does not settle on Semilight.
pub fn index_of(bytes: &[u8], preferred: &str) -> Option<u32> {
    let offsets = font_offsets(bytes)?;
    for (i, off) in offsets.into_iter().enumerate() {
        let names = face_names(bytes, off)?;
        if face_matches(&names, preferred) {
            return Some(i as u32);
        }
    }
    None
}

/// Like [`index_of`], but with two practical relaxations for system fonts:
/// an empty `name` selects face 0, and a single-font file whose name does
/// not match still contributes that one face rather than being skipped.
pub fn index_for_candidate(bytes: &[u8], name: &str) -> Option<u32> {
    if name.is_empty() {
        return Some(0);
    }
    if let Some(i) = index_of(bytes, name) {
        return Some(i);
    }
    match font_offsets(bytes) {
        Some(off) if off.len() == 1 => Some(0),
        _ => None,
    }
}

fn u16_be(data: &[u8], off: usize) -> Option<u16> {
    data.get(off..off + 2)?
        .try_into()
        .ok()
        .map(u16::from_be_bytes)
}

fn u32_be(data: &[u8], off: usize) -> Option<u32> {
    data.get(off..off + 4)?
        .try_into()
        .ok()
        .map(u32::from_be_bytes)
}

/// Starts of every font in a collection, or `[0]` for a single-font file.
fn font_offsets(data: &[u8]) -> Option<Vec<u32>> {
    if data.len() >= 4 && &data[..4] == b"ttcf" {
        let n = u32_be(data, 8)? as usize;
        // A real CJK collection is a handful of faces; anything huge is not
        // a font file we want to walk.
        if n == 0 || n > 64 {
            return None;
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            out.push(u32_be(data, 12 + i * 4)?);
        }
        Some(out)
    } else if data.len() >= 12 {
        Some(vec![0])
    } else {
        None
    }
}

struct FaceNames {
    family: Option<String>,
    subfamily: Option<String>,
    full: Option<String>,
    typo_family: Option<String>,
    typo_sub: Option<String>,
}

fn face_names(data: &[u8], font_offset: u32) -> Option<FaceNames> {
    let base = font_offset as usize;
    let num_tables = u16_be(data, base + 4)? as usize;
    let records = base + 12;
    let mut name_off = None;
    let mut name_len = 0usize;
    for i in 0..num_tables {
        let rec = records + i * 16;
        let tag = data.get(rec..rec + 4)?;
        if tag == b"name" {
            name_off = Some(u32_be(data, rec + 8)? as usize);
            name_len = u32_be(data, rec + 12)? as usize;
            break;
        }
    }
    let start = name_off?;
    let table = data.get(start..start.checked_add(name_len)?)?;
    parse_name_table(table)
}

fn parse_name_table(table: &[u8]) -> Option<FaceNames> {
    let count = u16_be(table, 2)? as usize;
    let string_off = u16_be(table, 4)? as usize;
    let mut family: Option<(u8, String)> = None;
    let mut subfamily: Option<(u8, String)> = None;
    let mut full: Option<(u8, String)> = None;
    let mut typo_family: Option<(u8, String)> = None;
    let mut typo_sub: Option<(u8, String)> = None;
    for i in 0..count {
        let rec = 6 + i * 12;
        let platform = u16_be(table, rec)?;
        let encoding = u16_be(table, rec + 2)?;
        let language = u16_be(table, rec + 4)?;
        let name_id = u16_be(table, rec + 6)?;
        let length = u16_be(table, rec + 8)? as usize;
        let offset = u16_be(table, rec + 10)? as usize;
        let slot = match name_id {
            1 => &mut family,
            2 => &mut subfamily,
            4 => &mut full,
            16 => &mut typo_family,
            17 => &mut typo_sub,
            _ => continue,
        };
        let rank = name_rank(platform, encoding, language)?;
        if slot.as_ref().is_some_and(|(r, _)| *r <= rank) {
            continue;
        }
        let bytes = table.get(string_off + offset..string_off + offset + length)?;
        let text = decode_name(platform, encoding, bytes)?;
        *slot = Some((rank, text));
    }
    Some(FaceNames {
        family: family.map(|(_, s)| s),
        subfamily: subfamily.map(|(_, s)| s),
        full: full.map(|(_, s)| s),
        typo_family: typo_family.map(|(_, s)| s),
        typo_sub: typo_sub.map(|(_, s)| s),
    })
}

/// Lower is better. We want the English Windows name when several exist.
fn name_rank(platform: u16, encoding: u16, language: u16) -> Option<u8> {
    match (platform, encoding) {
        // Microsoft Unicode BMP / full repertoire.
        (3, 1 | 10) => Some(if language == 0x0409 { 0 } else { 1 }),
        // Unicode platform.
        (0, _) => Some(2),
        _ => None,
    }
}

fn decode_name(platform: u16, encoding: u16, bytes: &[u8]) -> Option<String> {
    match (platform, encoding) {
        (3, 1 | 10) | (0, _) => {
            if bytes.len() % 2 != 0 {
                return None;
            }
            let units: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16(&units).ok()
        }
        _ => None,
    }
}

fn is_regular(sub: &str) -> bool {
    sub.is_empty() || sub.eq_ignore_ascii_case("regular") || sub.eq_ignore_ascii_case("normal")
}

fn face_matches(names: &FaceNames, preferred: &str) -> bool {
    let pref = preferred.trim();
    if pref.is_empty() {
        return false;
    }
    if names
        .full
        .as_deref()
        .is_some_and(|s| s.eq_ignore_ascii_case(pref))
    {
        return true;
    }
    let family = names
        .typo_family
        .as_deref()
        .or(names.family.as_deref())
        .unwrap_or("");
    let sub = names
        .typo_sub
        .as_deref()
        .or(names.subfamily.as_deref())
        .unwrap_or("");
    if family.eq_ignore_ascii_case(pref) && is_regular(sub) {
        return true;
    }
    if !sub.is_empty() {
        let composed = format!("{family} {sub}");
        if composed.eq_ignore_ascii_case(pref) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny sfnt-wrapped name table, enough for [`index_of`] to read.
    ///
    /// Table offsets inside a `.ttc` are from the start of the file, so a
    /// face that is not first must be emitted with its name table already
    /// relocated.
    fn font_with_names(faces: &[&[(u16, &str)]]) -> Vec<u8> {
        if faces.len() == 1 {
            return one_font(faces[0], 0);
        }
        let n = faces.len();
        let header_len = 12 + 4 * n;
        let sizes: Vec<usize> = faces.iter().map(|names| one_font(names, 0).len()).collect();
        let mut positions = Vec::new();
        let mut pos = header_len;
        for sz in &sizes {
            while pos % 4 != 0 {
                pos += 1;
            }
            positions.push(pos);
            pos += sz;
        }
        let mut out = b"ttcf".to_vec();
        out.extend_from_slice(&0x00010000u32.to_be_bytes());
        out.extend_from_slice(&(n as u32).to_be_bytes());
        for &p in &positions {
            out.extend_from_slice(&(p as u32).to_be_bytes());
        }
        for (i, names) in faces.iter().enumerate() {
            while out.len() < positions[i] {
                out.push(0);
            }
            out.extend(one_font(names, positions[i] as u32));
        }
        out
    }

    fn one_font(names: &[(u16, &str)], file_offset: u32) -> Vec<u8> {
        // name table: format 0, records, string store. Windows Unicode only.
        let mut records = Vec::new();
        let mut store = Vec::new();
        for &(id, text) in names {
            let encoded: Vec<u8> = text.encode_utf16().flat_map(|u| u.to_be_bytes()).collect();
            records.extend_from_slice(&3u16.to_be_bytes()); // platform
            records.extend_from_slice(&1u16.to_be_bytes()); // encoding
            records.extend_from_slice(&0x0409u16.to_be_bytes());
            records.extend_from_slice(&id.to_be_bytes());
            records.extend_from_slice(&(encoded.len() as u16).to_be_bytes());
            records.extend_from_slice(&(store.len() as u16).to_be_bytes());
            store.extend(encoded);
        }
        let mut name = Vec::new();
        name.extend_from_slice(&0u16.to_be_bytes()); // format
        name.extend_from_slice(&(names.len() as u16).to_be_bytes());
        name.extend_from_slice(&(6 + records.len() as u16).to_be_bytes());
        name.extend(records);
        name.extend(store);

        // Offset table + one table record, with the name table immediately
        // after the record (offset 28).
        let mut font = Vec::new();
        font.extend_from_slice(&0x00010000u32.to_be_bytes()); // scaler
        font.extend_from_slice(&1u16.to_be_bytes()); // numTables
        font.extend_from_slice(&16u16.to_be_bytes());
        font.extend_from_slice(&0u16.to_be_bytes());
        font.extend_from_slice(&0u16.to_be_bytes());
        font.extend_from_slice(b"name");
        font.extend_from_slice(&0u32.to_be_bytes()); // checksum unused
        font.extend_from_slice(&(28u32 + file_offset).to_be_bytes());
        font.extend_from_slice(&(name.len() as u32).to_be_bytes());
        font.extend(name);
        font
    }

    #[test]
    fn a_single_font_is_index_zero() {
        let bytes = font_with_names(&[&[
            (1, "Microsoft YaHei UI"),
            (2, "Regular"),
            (4, "Microsoft YaHei UI"),
        ]]);
        assert_eq!(index_of(&bytes, "Microsoft YaHei UI"), Some(0));
        assert_eq!(index_of(&bytes, "Microsoft YaHei"), None);
    }

    #[test]
    fn a_collection_picks_the_ui_face_not_the_body_face() {
        let bytes = font_with_names(&[
            &[
                (1, "Microsoft YaHei"),
                (2, "Regular"),
                (4, "Microsoft YaHei"),
            ],
            &[
                (1, "Microsoft YaHei UI"),
                (2, "Regular"),
                (4, "Microsoft YaHei UI"),
            ],
        ]);
        assert_eq!(index_of(&bytes, "Microsoft YaHei UI"), Some(1));
        assert_eq!(index_of(&bytes, "Microsoft YaHei"), Some(0));
    }

    #[test]
    fn a_family_name_does_not_match_a_non_regular_subfamily() {
        // YuGothR.ttc holds Yu Gothic UI Semilight under the typographic
        // family "Yu Gothic UI". Taking that face when we asked for Regular
        // would make the interface look washed out.
        let bytes = font_with_names(&[&[
            (1, "Yu Gothic UI Semilight"),
            (2, "Regular"),
            (4, "Yu Gothic UI Semilight"),
            (16, "Yu Gothic UI"),
            (17, "Semilight"),
        ]]);
        assert_eq!(index_of(&bytes, "Yu Gothic UI"), None);
        assert_eq!(index_of(&bytes, "Yu Gothic UI Semilight"), Some(0));
    }

    #[test]
    fn a_family_name_matches_the_regular_ui_face() {
        let bytes = font_with_names(&[&[
            (1, "Yu Gothic UI"),
            (2, "Regular"),
            (4, "Yu Gothic UI Regular"),
            (16, "Yu Gothic UI"),
            (17, "Regular"),
        ]]);
        assert_eq!(index_of(&bytes, "Yu Gothic UI"), Some(0));
        assert_eq!(index_of(&bytes, "Yu Gothic UI Regular"), Some(0));
    }

    #[test]
    fn a_missing_or_unrelated_name_is_none() {
        let bytes = font_with_names(&[&[(1, "Segoe UI"), (2, "Regular"), (4, "Segoe UI")]]);
        assert_eq!(index_of(&bytes, "Microsoft YaHei UI"), None);
        assert_eq!(index_of(&bytes, ""), None);
        assert_eq!(index_of(b"not a font", "Segoe UI"), None);
    }

    #[test]
    fn an_empty_candidate_name_selects_face_zero() {
        let bytes = font_with_names(&[&[(1, "Segoe UI"), (2, "Regular"), (4, "Segoe UI")]]);
        assert_eq!(index_for_candidate(&bytes, ""), Some(0));
    }

    #[test]
    fn a_single_font_still_loads_when_the_name_differs() {
        let bytes = font_with_names(&[&[(1, "Segoe UI"), (2, "Regular"), (4, "Segoe UI")]]);
        assert_eq!(index_for_candidate(&bytes, "Some Other Sans"), Some(0));
    }

    #[test]
    fn a_collection_is_not_guessed_when_the_name_is_absent() {
        let bytes = font_with_names(&[
            &[
                (1, "Microsoft YaHei"),
                (2, "Regular"),
                (4, "Microsoft YaHei"),
            ],
            &[
                (1, "Microsoft YaHei UI"),
                (2, "Regular"),
                (4, "Microsoft YaHei UI"),
            ],
        ]);
        assert_eq!(index_for_candidate(&bytes, "Meiryo UI"), None);
    }
}
