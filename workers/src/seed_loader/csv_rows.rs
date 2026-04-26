//! CSV row structs used by the per-table seed loaders.
//!
//! One struct per CSV file, with field names matching the CSV header
//! exactly so the csv crate's header-based deserialization picks the
//! right column without #[serde(rename=...)] noise.
//!
//! Types are chosen to match the CSV source: integer-looking columns
//! are `i32`; free text is `String`; bool-looking columns parse via
//! a small helper because the CSV emits lowercase "true" / "false"
//! that serde-csv's default bool parser handles, but some files use
//! the column name style that needs explicit deserialization.
//!
//! Column-type choices here matter for downstream loaders: e.g.
//! jump_heights.height_inches is a NUMERIC in the database but the
//! CSV stores plain integers, so the Rust side carries an i32 and
//! the loader casts at query time.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CountryRow {
    pub alpha2_code: String,
    pub alpha3_code: String,
    pub display_name: String,
    pub display_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct StateRow {
    pub country_alpha2_code: String,
    pub code: String,
    pub legacy_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BreedGroupRow {
    pub legacy_id: Option<i32>,
    pub group_number: i32,
    pub display_name: String,
    /// Pipe-delimited string in the CSV (e.g. "SN|SR|SS"). The loader
    /// splits on '|' into a Vec<String> for the TEXT[] column.
    pub registration_prefixes: String,
}

#[derive(Debug, Deserialize)]
pub struct BreedRow {
    pub legacy_id: Option<i32>,
    pub name: String,
    pub abbreviation: Option<String>,
    pub group_legacy_id: i32,
    pub is_giant: bool,
    pub is_three_quarters: bool,
    pub default_height_inches: Option<i32>,
    pub has_variety: bool,
    pub has_division: bool,
    pub display_order: Option<i32>,
    /// Access-era audit column (e.g. "01/26/17 00:00:00"). Ignored by
    /// the loader; present here only so serde does not reject the
    /// column.
    #[allow(dead_code)]
    pub source_date_added: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BreedVarietyRow {
    pub legacy_id: Option<i32>,
    pub breed_legacy_id: i32,
    pub name: String,
    pub display_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TitlePrefixRow {
    pub legacy_id: Option<i32>,
    pub code: String,
    pub long_name: Option<String>,
    pub sport_scope_code: Option<String>,
    pub sport_scope_description: Option<String>,
    pub source_organization: String,
}

#[derive(Debug, Deserialize)]
pub struct TitleSuffixRow {
    pub legacy_id: Option<i32>,
    pub code: String,
    pub long_name: Option<String>,
    pub sport_scope_code: Option<String>,
    pub sport_scope_description: Option<String>,
    pub source_organization: String,
    pub display_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct JumpHeightRow {
    #[serde(rename = "JumpID")]
    pub jump_id: i32,
    #[serde(rename = "Jump")]
    pub jump: i32,
    #[serde(rename = "Event")]
    pub event: String,
    #[serde(rename = "AssignOrder")]
    #[allow(dead_code)]
    pub assign_order: Option<i32>,
    #[serde(rename = "NewOrder")]
    pub new_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ObedienceExerciseRow {
    #[serde(rename = "ExerciseID")]
    pub exercise_id: i32,
    #[serde(rename = "Exercise")]
    pub exercise: String,
}

/// Wide-layout Box1..Box13 source for obedience_class_exercises.
/// Parsed into N normalized junction rows per CSV row by the loader.
#[derive(Debug, Deserialize)]
pub struct ObedienceClassExerciseRow {
    #[serde(rename = "ClassCode")]
    pub class_code: i32,
    #[serde(rename = "ClassPattern")]
    #[allow(dead_code)]
    pub class_pattern: i32,
    #[serde(rename = "Class")]
    pub class_name: String,
    #[serde(rename = "Box1")]
    pub box1: Option<String>,
    #[serde(rename = "Box2")]
    pub box2: Option<String>,
    #[serde(rename = "Box3")]
    pub box3: Option<String>,
    #[serde(rename = "Box4")]
    pub box4: Option<String>,
    #[serde(rename = "Box5")]
    pub box5: Option<String>,
    #[serde(rename = "Box6")]
    pub box6: Option<String>,
    #[serde(rename = "Box7")]
    pub box7: Option<String>,
    #[serde(rename = "Box8")]
    pub box8: Option<String>,
    #[serde(rename = "Box9")]
    pub box9: Option<String>,
    #[serde(rename = "Box10")]
    pub box10: Option<String>,
    #[serde(rename = "Box11")]
    pub box11: Option<String>,
    #[serde(rename = "Box12")]
    pub box12: Option<String>,
    #[serde(rename = "Box13")]
    pub box13: Option<String>,
}

impl ObedienceClassExerciseRow {
    /// Return the 13 boxes as a (display_order, cell_text) sequence,
    /// filtering out empty or whitespace-only cells. display_order is
    /// the 1-based box position.
    pub fn boxes(&self) -> Vec<(i32, String)> {
        let raw = [
            &self.box1,
            &self.box2,
            &self.box3,
            &self.box4,
            &self.box5,
            &self.box6,
            &self.box7,
            &self.box8,
            &self.box9,
            &self.box10,
            &self.box11,
            &self.box12,
            &self.box13,
        ];
        raw.iter()
            .enumerate()
            .filter_map(|(idx, cell)| match cell {
                Some(text) => {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some((idx as i32 + 1, text.to_string()))
                    }
                }
                None => None,
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct OtchPointsRow {
    pub legacy_id: Option<i32>,
    pub class_name: String,
    pub entries_min: i32,
    pub entries_max: i32,
    pub first_place_points: i32,
    pub second_place_points: i32,
    pub third_place_points: i32,
    pub fourth_place_points: i32,
}

#[derive(Debug, Deserialize)]
pub struct OmPointsRow {
    pub legacy_id: Option<i32>,
    /// Kept as String so the NUMERIC(4,1) bind below can go straight to
    /// sqlx without depending on a decimal crate.
    pub score: String,
    pub om_points: i32,
}

#[derive(Debug, Deserialize)]
pub struct RallyRachPointsRow {
    pub legacy_id: Option<i32>,
    pub score: String,
    pub rach_points: i32,
}

#[derive(Debug, Deserialize)]
pub struct SportTimeDefaultRow {
    pub legacy_id: Option<i32>,
    pub sport_or_event: String,
    pub minutes_per_dog: String,
    pub class_change_seconds: i32,
    pub event_change_seconds: i32,
}

/// One AKC-recognized combined-award grouping. Drives both the
/// additional-entry fee discount logic (per Deborah's Q4: any double
/// or triple Q in B classes earns the discount) and, for groups with
/// a non-NULL `award_type`, the per-trial combined-award computation.
///
/// `award_type` is left empty in the CSV for title-progression paths
/// (RAE, RACH) that do NOT produce a per-trial ribbon. The csv crate
/// deserializes empty fields into `Option<String>::None`, which the
/// loader then binds as SQL NULL.
#[derive(Debug, Deserialize)]
pub struct CombinedAwardGroupRow {
    pub code: String,
    pub sport: String,
    pub display_name: String,
    pub award_type: Option<String>,
    pub is_discount_eligible: bool,
    pub regulation_citation: Option<String>,
}

/// One canonical-class membership in one combined-award group.
/// Joins `combined_award_groups.code` to `canonical_classes.code`.
/// `is_required_for_award` is TRUE for every CHECKPOINT 2 seed row;
/// the schema retains the flag for future groups whose semantics
/// require optional contributors.
#[derive(Debug, Deserialize)]
pub struct CombinedAwardGroupClassRow {
    pub group_code: String,
    pub canonical_class_code: String,
    pub is_required_for_award: bool,
}
