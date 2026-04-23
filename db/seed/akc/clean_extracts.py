#!/usr/bin/env python3
"""
Clean and transform AKC reference data extracted from ObedienceData.mde
into QTrial-idiomatic seed CSVs.

Source: Deborah Pruyn's Obedience Solution by Lab Tested Databases,
        ObedienceData.accde, data vintage through 2018-2019 (multiple AKC rule revisions reflected).

Output: CSVs ready to commit to db/seed/akc/ and consume via sqlx migrations.

Transformations applied:
- Column names renamed to snake_case, matching DATA_MODEL.md conventions
- Trailing whitespace stripped from all string fields
- Boolean-as-integer (0/1) normalized to lowercase true/false for CSV clarity
- Access's autoincrement IDs preserved as legacy_id for migration traceback
- Countries: AKC codes preserved as legacy_akc_code alongside standard ISO codes
- States: StateID dropped (arbitrary), 2-letter code becomes natural key
"""

import csv
import os
from pathlib import Path

SRC = Path("/home/claude/db_seed_akc_v2")
OUT = Path("/home/claude/db_seed_akc_v2")


def strip_trailing_whitespace(value):
    """Access exports often have trailing spaces on text fields. Strip them."""
    if isinstance(value, str):
        return value.strip()
    return value


def bool_from_access(value):
    """Access boolean is 0 or 1; normalize to CSV-friendly true/false strings."""
    if value in ("1", 1, True):
        return "true"
    if value in ("0", 0, False, "", None):
        return "false"
    return str(value)


def read_access_csv(path):
    """Read an mdb-export CSV with proper quoted-field handling."""
    with open(path, newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        return [row for row in reader]


def write_csv(path, fieldnames, rows):
    """Write a clean CSV with consistent quoting."""
    with open(path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames, quoting=csv.QUOTE_MINIMAL)
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


# ---------------------------------------------------------------------------
# breed_groups
# ---------------------------------------------------------------------------
# Source: tblAKCGroups
# Target columns: legacy_id, group_number, display_name, registration_prefixes
# The legacy schema stores up to 4 registration prefix codes in separate columns;
# QTrial stores them as an array-serialized field (Postgres TEXT[] via pipe-
# delimited string, expanded by the migration).

def clean_breed_groups():
    src_rows = read_access_csv(SRC / "raw_tblAKCGroups.csv")
    out_rows = []
    for r in src_rows:
        group_id = int(r["AKCGroup"])
        name = strip_trailing_whitespace(r.get("Name", ""))
        # Group 0 has empty name in source - skip it; it's a sentinel row.
        if not name:
            continue
        prefixes = []
        for key in ("Prefix1", "Prefix2", "Prefix3", "Prefix4"):
            val = strip_trailing_whitespace(r.get(key, ""))
            if val:
                prefixes.append(val)
        out_rows.append({
            "legacy_id": group_id,
            "group_number": group_id,
            "display_name": name,
            "registration_prefixes": "|".join(prefixes),
        })
    fieldnames = ["legacy_id", "group_number", "display_name", "registration_prefixes"]
    write_csv(OUT / "breed_groups.csv", fieldnames, out_rows)
    print(f"  breed_groups.csv:    {len(out_rows)} rows")


# ---------------------------------------------------------------------------
# breeds
# ---------------------------------------------------------------------------
# Source: tblBreeds
# Target columns match DATA_MODEL.md §8 breeds table

def clean_breeds():
    src_rows = read_access_csv(SRC / "raw_tblBreeds.csv")
    out_rows = []
    for r in src_rows:
        name = strip_trailing_whitespace(r["Breed"])
        if not name:
            continue
        out_rows.append({
            "legacy_id": int(r["BreedID"]),
            "name": name,
            "abbreviation": strip_trailing_whitespace(r.get("BreedAbbrev", "")) or name,
            "group_legacy_id": int(r["AKCGroup"]) if r.get("AKCGroup") else None,
            "is_giant": bool_from_access(r.get("Giant")),
            "is_three_quarters": bool_from_access(r.get("ThreeQuarters")),
            "default_height_inches": int(r["Height"]) if r.get("Height") else None,
            "has_variety": bool_from_access(r.get("HasVariety")),
            "has_division": bool_from_access(r.get("HasDivision")),
            "display_order": int(r["Order"]) if r.get("Order") else None,
            "source_date_added": strip_trailing_whitespace(r.get("DateAdded", "")),
        })
    fieldnames = [
        "legacy_id", "name", "abbreviation", "group_legacy_id",
        "is_giant", "is_three_quarters", "default_height_inches",
        "has_variety", "has_division", "display_order", "source_date_added",
    ]
    write_csv(OUT / "breeds.csv", fieldnames, out_rows)
    print(f"  breeds.csv:          {len(out_rows)} rows")


# ---------------------------------------------------------------------------
# breed_varieties
# ---------------------------------------------------------------------------
# Source: tblBreedVariety

def clean_breed_varieties():
    src_rows = read_access_csv(SRC / "raw_tblBreedVariety.csv")
    out_rows = []
    for r in src_rows:
        out_rows.append({
            "legacy_id": int(r["VarietyID"]),
            "breed_legacy_id": int(r["BreedID"]),
            "name": strip_trailing_whitespace(r["Variety"]),
            "display_order": int(r["VarietyOrder"]) if r.get("VarietyOrder") else None,
        })
    fieldnames = ["legacy_id", "breed_legacy_id", "name", "display_order"]
    write_csv(OUT / "breed_varieties.csv", fieldnames, out_rows)
    print(f"  breed_varieties.csv: {len(out_rows)} rows")


# ---------------------------------------------------------------------------
# title_prefixes
# ---------------------------------------------------------------------------

def clean_title_prefixes():
    """Generate title_prefixes.csv from AKC-authoritative source.

    This function produces the AKC-published prefix title catalog, preserving
    legacy_id values from Deborah's ObedienceData.accde where titles match,
    and assigning new IDs in the 1000+ range for AKC titles absent from her
    catalog. See README.md for the rationale and AKC_overrides_added.csv for
    the full audit trail of additions.

    The authoritative list is embedded at module bottom in AKC_PREFIX_TITLES
    for deterministic, self-contained seeds. Regenerating the AKC list from
    www.akc.org/sports/titles-and-abbreviations/ is a separate workflow
    documented in README.md.
    """
    # Legacy ID mapping from Deborah's data - preserve these for migration continuity
    legacy_id_by_code = {}
    src_rows = read_access_csv(SRC / "raw_tblAKCTitlesPrefix.csv")
    for r in src_rows:
        code = strip_trailing_whitespace(r["AKCPrefix"])
        if code:
            legacy_id_by_code[code] = int(r["AKCPrefixID"])

    out_rows = []
    next_new_id = 1001  # IDs in 1000+ range identify AKC titles not in Deborah's catalog
    for code, long_name, scope_code, scope_desc in AKC_PREFIX_TITLES:
        if code in legacy_id_by_code:
            legacy_id = legacy_id_by_code[code]
        else:
            legacy_id = next_new_id
            next_new_id += 1
        out_rows.append({
            "legacy_id": legacy_id,
            "code": code,
            "long_name": long_name,
            "sport_scope_code": scope_code,
            "sport_scope_description": scope_desc,
            "source_organization": "AKC",
        })

    out_rows.sort(key=lambda r: r["legacy_id"])
    fieldnames = ["legacy_id", "code", "long_name", "sport_scope_code",
                  "sport_scope_description", "source_organization"]
    write_csv(OUT / "title_prefixes.csv", fieldnames, out_rows)
    print(f"  title_prefixes.csv:  {len(out_rows)} rows "
          f"({sum(1 for r in out_rows if r['legacy_id'] < 1000)} from Deborah, "
          f"{sum(1 for r in out_rows if r['legacy_id'] >= 1000)} from AKC)")


# ---------------------------------------------------------------------------
# title_suffixes
# ---------------------------------------------------------------------------
# The Type column in Deborah's source is the sport-scope indicator. Per our
# analysis the single-letter codes map to:
#   O=Obedience, R=Rally, A=Agility Standard, J=Agility Jumpers, P=Preferred
#   Agility, Q=Preferred Agility Jumpers, Y=Agility FAST, Z=Preferred FAST,
#   T=Tracking, U/S=T2B variants, F=Field/Hunt, E=Earthdog, H=Herding,
#   C=Coursing, V=Versatility, X=Companion, empty=Compound titles.
# We extend this with post-Lab-Tested-Databases sports:
#   W=Scent Work, K=Trick Dog, D=Therapy Dog, I=FIT Dog, B=Fetch, G=Virtual,
#   N=Conformation, M=Coonhound, L=Lure Coursing, FCAT=Fast CAT, TT=Temperament
# Full definitions are embedded in AKC_SUFFIX_TITLES at module bottom.

SPORT_SCOPE_DESCRIPTIONS = {
    "O": "Obedience",
    "R": "Rally",
    "A": "Agility Standard",
    "J": "Agility Jumpers With Weaves",
    "P": "Preferred Agility Standard",
    "Q": "Preferred Agility Jumpers",
    "Y": "Agility FAST",
    "Z": "Preferred Agility FAST",
    "T": "Tracking",
    "U": "Time 2 Beat",
    "S": "Preferred Time 2 Beat",
    "F": "Field / Hunt Test",
    "E": "Earthdog",
    "H": "Herding",
    "C": "Coursing",
    "V": "Versatility (multi-sport)",
    "X": "Companion (CGC, THD, etc.)",
    "W": "Scent Work",
    "K": "Trick Dog",
    "D": "Therapy Dog",
    "I": "FIT Dog",
    "B": "Fetch",
    "G": "Virtual (pandemic-era remote titles)",
    "N": "Conformation",
    "M": "Coonhound",
    "L": "Lure Coursing",
    "BH": "Barn Hunt (AKC-recognized via Barn Hunt Association)",
    "FCAT": "Fast CAT",
    "TT": "Temperament Test",
    "": "Compound / legacy (combines multiple single-sport titles)",
}


def clean_title_suffixes():
    """Generate title_suffixes.csv from AKC-authoritative source.

    Like clean_title_prefixes, preserves legacy_id from Deborah's data when
    codes match, uses 1000+ range for new AKC titles. Also preserves legacy
    compound titles (UDTD, UDVST, etc.) that AKC no longer publishes as
    single codes but which appear in historical dog registered names.
    """
    legacy_id_by_code = {}
    legacy_display_order_by_code = {}
    src_rows = read_access_csv(SRC / "raw_tblAKCTitlesSuffix.csv")
    for r in src_rows:
        code = strip_trailing_whitespace(r["AKCSuffix"])
        if code:
            # Source has typo: AKCSufficID instead of AKCSuffixID
            legacy_id_by_code[code] = int(r["AKCSufficID"])
            if r.get("SuffixOrder"):
                legacy_display_order_by_code[code] = int(r["SuffixOrder"])

    # Some AKC-recognized titles are issued by non-AKC organizations but appear
    # in AKC trial catalogs per the Title Recognition Program. We track the real
    # issuing body in source_organization so the catalog can render it correctly.
    # Currently: Barn Hunt titles (scope code "BH") are issued by Barn Hunt Assoc.
    # See AKC "Other Recognized Titles" section: https://www.akc.org/sports/titles-and-abbreviations/
    source_organization_by_scope = {
        "BH": "Barn Hunt Association",
    }

    out_rows = []
    next_new_id = 1001
    for code, long_name, scope_code, scope_desc in AKC_SUFFIX_TITLES:
        if code in legacy_id_by_code:
            legacy_id = legacy_id_by_code[code]
            display_order = legacy_display_order_by_code.get(code)
        else:
            legacy_id = next_new_id
            next_new_id += 1
            display_order = None  # new codes get blank; catalog can sort alphabetically
        out_rows.append({
            "legacy_id": legacy_id,
            "code": code,
            "long_name": long_name,
            "sport_scope_code": scope_code,
            "sport_scope_description": scope_desc,
            "source_organization": source_organization_by_scope.get(scope_code, "AKC"),
            "display_order": display_order,
        })

    # Preserve legacy compound titles (UDTD, UDVST, etc.) from Deborah's data
    for code, long_name, scope_code, scope_desc in LEGACY_COMPOUND_SUFFIX_TITLES:
        if code in legacy_id_by_code:
            out_rows.append({
                "legacy_id": legacy_id_by_code[code],
                "code": code,
                "long_name": long_name,
                "sport_scope_code": scope_code,
                "sport_scope_description": scope_desc,
                "source_organization": "AKC",
                "display_order": legacy_display_order_by_code.get(code),
            })

    out_rows.sort(key=lambda r: r["legacy_id"])
    fieldnames = ["legacy_id", "code", "long_name", "sport_scope_code",
                  "sport_scope_description", "source_organization", "display_order"]
    write_csv(OUT / "title_suffixes.csv", fieldnames, out_rows)
    from_deborah = sum(1 for r in out_rows if r['legacy_id'] < 1000)
    from_akc = sum(1 for r in out_rows if r['legacy_id'] >= 1000)
    print(f"  title_suffixes.csv:  {len(out_rows)} rows "
          f"({from_deborah} from Deborah, {from_akc} from AKC)")


# ---------------------------------------------------------------------------
# countries - SPECIAL CASE
# ---------------------------------------------------------------------------
# The AKC codes in the source are NOT ISO 3166-1 alpha-3 codes. Examples:
#   USA (matches ISO alpha-3 USA)
#   USR (Russia - ISO is RUS)
#   UKG (United Kingdom - ISO is GBR)
#   WGR (West Germany - no longer exists)
#   VLA (duplicate entry for Venezuela - ISO is VEN)
#
# For QTrial, we emit TWO files:
#   1. countries.csv - ISO 3166-1 (alpha-2 and alpha-3) standard data. This
#      is the authoritative countries list QTrial uses.
#   2. legacy_akc_country_codes.csv - AKC's legacy codes mapped to ISO alpha-2
#      for migration purposes. When importing historical dog data that
#      references "UKG", the migration layer resolves that to "GB".
#
# The ISO 3166-1 data is embedded here rather than fetched externally so the
# seed is self-contained and deterministic. This list is stable and only
# changes when countries form/dissolve (rare).

# ISO 3166-1 alpha-2 code, alpha-3 code, English short name
# Source: ISO 3166-1 Maintenance Agency published list, subset covering all
# countries that could plausibly appear in AKC-registered dog ownership data.
# Full 249-entry list; trimmed inline comments for brevity.
ISO_3166_1 = [
    # Format: (alpha2, alpha3, name)
    ("AD", "AND", "Andorra"),
    ("AE", "ARE", "United Arab Emirates"),
    ("AF", "AFG", "Afghanistan"),
    ("AG", "ATG", "Antigua and Barbuda"),
    ("AI", "AIA", "Anguilla"),
    ("AL", "ALB", "Albania"),
    ("AM", "ARM", "Armenia"),
    ("AO", "AGO", "Angola"),
    ("AR", "ARG", "Argentina"),
    ("AS", "ASM", "American Samoa"),
    ("AT", "AUT", "Austria"),
    ("AU", "AUS", "Australia"),
    ("AW", "ABW", "Aruba"),
    ("AZ", "AZE", "Azerbaijan"),
    ("BA", "BIH", "Bosnia and Herzegovina"),
    ("BB", "BRB", "Barbados"),
    ("BD", "BGD", "Bangladesh"),
    ("BE", "BEL", "Belgium"),
    ("BF", "BFA", "Burkina Faso"),
    ("BG", "BGR", "Bulgaria"),
    ("BH", "BHR", "Bahrain"),
    ("BI", "BDI", "Burundi"),
    ("BJ", "BEN", "Benin"),
    ("BM", "BMU", "Bermuda"),
    ("BN", "BRN", "Brunei Darussalam"),
    ("BO", "BOL", "Bolivia"),
    ("BR", "BRA", "Brazil"),
    ("BS", "BHS", "Bahamas"),
    ("BT", "BTN", "Bhutan"),
    ("BW", "BWA", "Botswana"),
    ("BY", "BLR", "Belarus"),
    ("BZ", "BLZ", "Belize"),
    ("CA", "CAN", "Canada"),
    ("CD", "COD", "Congo, Democratic Republic of the"),
    ("CF", "CAF", "Central African Republic"),
    ("CG", "COG", "Congo"),
    ("CH", "CHE", "Switzerland"),
    ("CI", "CIV", "Cote d'Ivoire"),
    ("CK", "COK", "Cook Islands"),
    ("CL", "CHL", "Chile"),
    ("CM", "CMR", "Cameroon"),
    ("CN", "CHN", "China"),
    ("CO", "COL", "Colombia"),
    ("CR", "CRI", "Costa Rica"),
    ("CU", "CUB", "Cuba"),
    ("CV", "CPV", "Cabo Verde"),
    ("CY", "CYP", "Cyprus"),
    ("CZ", "CZE", "Czechia"),
    ("DE", "DEU", "Germany"),
    ("DJ", "DJI", "Djibouti"),
    ("DK", "DNK", "Denmark"),
    ("DM", "DMA", "Dominica"),
    ("DO", "DOM", "Dominican Republic"),
    ("DZ", "DZA", "Algeria"),
    ("EC", "ECU", "Ecuador"),
    ("EE", "EST", "Estonia"),
    ("EG", "EGY", "Egypt"),
    ("ER", "ERI", "Eritrea"),
    ("ES", "ESP", "Spain"),
    ("ET", "ETH", "Ethiopia"),
    ("FI", "FIN", "Finland"),
    ("FJ", "FJI", "Fiji"),
    ("FK", "FLK", "Falkland Islands"),
    ("FM", "FSM", "Micronesia"),
    ("FO", "FRO", "Faroe Islands"),
    ("FR", "FRA", "France"),
    ("GA", "GAB", "Gabon"),
    ("GB", "GBR", "United Kingdom"),
    ("GD", "GRD", "Grenada"),
    ("GE", "GEO", "Georgia"),
    ("GF", "GUF", "French Guiana"),
    ("GH", "GHA", "Ghana"),
    ("GI", "GIB", "Gibraltar"),
    ("GL", "GRL", "Greenland"),
    ("GM", "GMB", "Gambia"),
    ("GN", "GIN", "Guinea"),
    ("GP", "GLP", "Guadeloupe"),
    ("GQ", "GNQ", "Equatorial Guinea"),
    ("GR", "GRC", "Greece"),
    ("GT", "GTM", "Guatemala"),
    ("GU", "GUM", "Guam"),
    ("GY", "GUY", "Guyana"),
    ("HK", "HKG", "Hong Kong"),
    ("HN", "HND", "Honduras"),
    ("HR", "HRV", "Croatia"),
    ("HT", "HTI", "Haiti"),
    ("HU", "HUN", "Hungary"),
    ("ID", "IDN", "Indonesia"),
    ("IE", "IRL", "Ireland"),
    ("IL", "ISR", "Israel"),
    ("IN", "IND", "India"),
    ("IQ", "IRQ", "Iraq"),
    ("IR", "IRN", "Iran"),
    ("IS", "ISL", "Iceland"),
    ("IT", "ITA", "Italy"),
    ("JM", "JAM", "Jamaica"),
    ("JO", "JOR", "Jordan"),
    ("JP", "JPN", "Japan"),
    ("KE", "KEN", "Kenya"),
    ("KG", "KGZ", "Kyrgyzstan"),
    ("KH", "KHM", "Cambodia"),
    ("KI", "KIR", "Kiribati"),
    ("KM", "COM", "Comoros"),
    ("KN", "KNA", "Saint Kitts and Nevis"),
    ("KP", "PRK", "Korea, Democratic People's Republic of"),
    ("KR", "KOR", "Korea, Republic of"),
    ("KW", "KWT", "Kuwait"),
    ("KY", "CYM", "Cayman Islands"),
    ("KZ", "KAZ", "Kazakhstan"),
    ("LA", "LAO", "Lao People's Democratic Republic"),
    ("LB", "LBN", "Lebanon"),
    ("LC", "LCA", "Saint Lucia"),
    ("LI", "LIE", "Liechtenstein"),
    ("LK", "LKA", "Sri Lanka"),
    ("LR", "LBR", "Liberia"),
    ("LS", "LSO", "Lesotho"),
    ("LT", "LTU", "Lithuania"),
    ("LU", "LUX", "Luxembourg"),
    ("LV", "LVA", "Latvia"),
    ("LY", "LBY", "Libya"),
    ("MA", "MAR", "Morocco"),
    ("MC", "MCO", "Monaco"),
    ("MD", "MDA", "Moldova"),
    ("ME", "MNE", "Montenegro"),
    ("MG", "MDG", "Madagascar"),
    ("MH", "MHL", "Marshall Islands"),
    ("MK", "MKD", "North Macedonia"),
    ("ML", "MLI", "Mali"),
    ("MM", "MMR", "Myanmar"),
    ("MN", "MNG", "Mongolia"),
    ("MO", "MAC", "Macao"),
    ("MQ", "MTQ", "Martinique"),
    ("MR", "MRT", "Mauritania"),
    ("MS", "MSR", "Montserrat"),
    ("MT", "MLT", "Malta"),
    ("MU", "MUS", "Mauritius"),
    ("MV", "MDV", "Maldives"),
    ("MW", "MWI", "Malawi"),
    ("MX", "MEX", "Mexico"),
    ("MY", "MYS", "Malaysia"),
    ("MZ", "MOZ", "Mozambique"),
    ("NA", "NAM", "Namibia"),
    ("NC", "NCL", "New Caledonia"),
    ("NE", "NER", "Niger"),
    ("NG", "NGA", "Nigeria"),
    ("NI", "NIC", "Nicaragua"),
    ("NL", "NLD", "Netherlands"),
    ("NO", "NOR", "Norway"),
    ("NP", "NPL", "Nepal"),
    ("NR", "NRU", "Nauru"),
    ("NZ", "NZL", "New Zealand"),
    ("OM", "OMN", "Oman"),
    ("PA", "PAN", "Panama"),
    ("PE", "PER", "Peru"),
    ("PF", "PYF", "French Polynesia"),
    ("PG", "PNG", "Papua New Guinea"),
    ("PH", "PHL", "Philippines"),
    ("PK", "PAK", "Pakistan"),
    ("PL", "POL", "Poland"),
    ("PR", "PRI", "Puerto Rico"),
    ("PT", "PRT", "Portugal"),
    ("PW", "PLW", "Palau"),
    ("PY", "PRY", "Paraguay"),
    ("QA", "QAT", "Qatar"),
    ("RE", "REU", "Reunion"),
    ("RO", "ROU", "Romania"),
    ("RS", "SRB", "Serbia"),
    ("RU", "RUS", "Russian Federation"),
    ("RW", "RWA", "Rwanda"),
    ("SA", "SAU", "Saudi Arabia"),
    ("SB", "SLB", "Solomon Islands"),
    ("SC", "SYC", "Seychelles"),
    ("SD", "SDN", "Sudan"),
    ("SE", "SWE", "Sweden"),
    ("SG", "SGP", "Singapore"),
    ("SI", "SVN", "Slovenia"),
    ("SK", "SVK", "Slovakia"),
    ("SL", "SLE", "Sierra Leone"),
    ("SM", "SMR", "San Marino"),
    ("SN", "SEN", "Senegal"),
    ("SO", "SOM", "Somalia"),
    ("SR", "SUR", "Suriname"),
    ("SS", "SSD", "South Sudan"),
    ("SV", "SLV", "El Salvador"),
    ("SY", "SYR", "Syrian Arab Republic"),
    ("SZ", "SWZ", "Eswatini"),
    ("TC", "TCA", "Turks and Caicos Islands"),
    ("TD", "TCD", "Chad"),
    ("TG", "TGO", "Togo"),
    ("TH", "THA", "Thailand"),
    ("TJ", "TJK", "Tajikistan"),
    ("TL", "TLS", "Timor-Leste"),
    ("TM", "TKM", "Turkmenistan"),
    ("TN", "TUN", "Tunisia"),
    ("TO", "TON", "Tonga"),
    ("TR", "TUR", "Turkiye"),
    ("TT", "TTO", "Trinidad and Tobago"),
    ("TW", "TWN", "Taiwan"),
    ("TZ", "TZA", "Tanzania"),
    ("UA", "UKR", "Ukraine"),
    ("UG", "UGA", "Uganda"),
    ("US", "USA", "United States of America"),
    ("UY", "URY", "Uruguay"),
    ("UZ", "UZB", "Uzbekistan"),
    ("VA", "VAT", "Holy See (Vatican)"),
    ("VC", "VCT", "Saint Vincent and the Grenadines"),
    ("VE", "VEN", "Venezuela"),
    ("VG", "VGB", "Virgin Islands, British"),
    ("VI", "VIR", "Virgin Islands, U.S."),
    ("VN", "VNM", "Viet Nam"),
    ("VU", "VUT", "Vanuatu"),
    ("WS", "WSM", "Samoa"),
    ("YE", "YEM", "Yemen"),
    ("ZA", "ZAF", "South Africa"),
    ("ZM", "ZMB", "Zambia"),
    ("ZW", "ZWE", "Zimbabwe"),
]

# AKC legacy code mappings. Derived from tblCountries inspection plus
# reasonable mapping judgments for historical codes. For codes where no
# successor country exists (dissolved multinational federations, regional
# designations), iso_alpha2_code is None and needs_manual_resolution=true
# so a human can decide case-by-case.
AKC_TO_ISO_ALPHA2 = {
    # Direct matches to current countries
    "USA": "US", "CAN": "CA", "UKG": "GB", "UKR": "UA",
    "USR": "RU",  # "USR" in AKC data = Russia (USSR-era code)
    "RUS": "RU",  # Russia, standard
    "VEN": "VE", "VLA": "VE",  # duplicate Venezuela entries in source
    "ZIM": "ZW", "ADR": "AD", "ALG": "DZ", "AMS": "AS", "ARB": "AW",
    "ARG": "AR", "AST": "AT", "AUS": "AU",
    "BAH": "BS", "BAN": "BD", "BAR": "BB",
    "BVI": "VG",  # British Virgin Islands -> ISO VG
    "TKY": "TR",  # Turkey -> TR (ISO 3166-1 spelling is now Turkiye)
    "TRI": "TT",  # Trinidad -> Trinidad and Tobago
    "SUR": "SR",  # Surinam/Suriname
    "GSY": "GG",  # Guernsey (ISO 3166-1 assigns GG)
    "SCT": "GB",  # Scotland -> part of United Kingdom
    "IRE": "IE",  # Ireland (Republic)
    "CZR": "CZ",  # Czech Republic -> Czechia
    "KOR": "KR", "SKR": "KR",  # South Korea
    "NKR": "KP",  # North Korea
    "RCH": "TW",  # "Republic of China" in AKC context = Taiwan
    "RGA": "GE",  # "Republic of Geo" = Republic of Georgia
    "RSL": "SI",  # Republic of Slovenia
    "RUM": "RO",  # Rumania (archaic spelling of Romania)
    "MIQ": "PM",  # Miquelon -> Saint Pierre and Miquelon
    "NAN": None,  # Netherlands Antilles dissolved 2010; now CW, SX, BQ, etc.
    # Regional / dissolved / multinational entities - no successor mapping
    "WAF": None,  # "West Africa" - regional designation
    "WIN": None,  # "West Indies" - regional designation
    "BWI": None,  # "British West Indies" - no longer exists as entity
    "FWI": None,  # "French West Indies" - regional designation
    "GWI": None,  # "Grenada West Indies" - probably just GD
    "EAF": None,  # "East Africa" - regional
    "WGR": "DE",  # West Germany -> reunified Germany
    "EGR": "DE",  # East Germany -> reunified Germany
    "YUG": None,  # Yugoslavia dissolved - successor states vary
    "CSF": None,  # Czech and Slovak Federative Republic dissolved 1993
    "CZE": None,  # Czechoslovakia dissolved 1993
    "SFR": None,  # "Slovok Federate" - likely Czechoslovakia-era
    "CZN": None,  # "Canal Zone" - Panama Canal Zone, dissolved 1979
    "NVN": None,  # North Vietnam dissolved 1976
    "SVN": None,  # South Vietnam dissolved 1975
    "AZR": None,  # Azores - region of Portugal (PT), not its own ISO code
}


def clean_countries():
    """Emit ISO 3166-1 countries table."""
    out_rows = []
    for i, (alpha2, alpha3, name) in enumerate(ISO_3166_1, start=1):
        out_rows.append({
            "alpha2_code": alpha2,
            "alpha3_code": alpha3,
            "display_name": name,
            "display_order": i,
        })
    fieldnames = ["alpha2_code", "alpha3_code", "display_name", "display_order"]
    write_csv(OUT / "countries.csv", fieldnames, out_rows)
    print(f"  countries.csv:       {len(out_rows)} rows (ISO 3166-1 standard)")


def clean_akc_country_mappings():
    """Emit legacy AKC code -> ISO alpha-2 mapping for migration reference."""
    src_rows = read_access_csv(SRC / "raw_tblCountries.csv")
    out_rows = []
    for r in src_rows:
        akc_code = strip_trailing_whitespace(r["Abbrev"])
        akc_name = strip_trailing_whitespace(r["Country"])
        iso_alpha2 = AKC_TO_ISO_ALPHA2.get(akc_code)
        # If not in our explicit mapping, try to match by name
        if iso_alpha2 is None and akc_code not in AKC_TO_ISO_ALPHA2:
            for a2, _, name in ISO_3166_1:
                if name.lower() == akc_name.lower():
                    iso_alpha2 = a2
                    break
        out_rows.append({
            "legacy_akc_code": akc_code,
            "legacy_akc_name": akc_name,
            "iso_alpha2_code": iso_alpha2 or "",
            "needs_manual_resolution": "true" if iso_alpha2 is None else "false",
        })
    fieldnames = ["legacy_akc_code", "legacy_akc_name", "iso_alpha2_code", "needs_manual_resolution"]
    write_csv(OUT / "legacy_akc_country_codes.csv", fieldnames, out_rows)
    unresolved = sum(1 for r in out_rows if r["needs_manual_resolution"] == "true")
    print(f"  legacy_akc_country_codes.csv: {len(out_rows)} rows ({unresolved} needing manual resolution)")


# ---------------------------------------------------------------------------
# states / provinces
# ---------------------------------------------------------------------------
# Source tblStates mixes US states + DC + Canadian provinces in one table.
# We split them by country since they have different governance (US states
# are ISO 3166-2:US, Canadian provinces are ISO 3166-2:CA).

US_STATES = {
    # 50 states + DC
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA",
    "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
    "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
    "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
    "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
    "DC",
}

CA_PROVINCES = {
    "AB", "BC", "MB", "NB", "NL", "NF", "NS", "NT", "NU", "ON",
    "PE", "QC", "SK", "YT",
}


def clean_states():
    src_rows = read_access_csv(SRC / "raw_tblStates.csv")
    out_rows = []
    for r in src_rows:
        code = strip_trailing_whitespace(r["State"])
        if not code:
            continue
        if code in US_STATES:
            country_code = "US"
        elif code in CA_PROVINCES:
            country_code = "CA"
        else:
            country_code = "??"  # unknown; flagged
        out_rows.append({
            "country_alpha2_code": country_code,
            "code": code,
            "legacy_id": int(r["StateID"]),
        })
    # Sort for stable output: country first, then code
    out_rows.sort(key=lambda r: (r["country_alpha2_code"], r["code"]))
    fieldnames = ["country_alpha2_code", "code", "legacy_id"]
    write_csv(OUT / "states.csv", fieldnames, out_rows)
    print(f"  states.csv:          {len(out_rows)} rows")


# ===========================================================================
# v2 ADDITIONS - new tables present in ObedienceData.accde
# ===========================================================================
#
# These tables were not in the older ObedienceData.mde but are present in
# the newer ObedienceData.accde. They reflect AKC rule changes from 2018-
# 2019 and Lab Tested Databases' expansion to track non-AKC titles.

# ---------------------------------------------------------------------------
# canonical_classes (from tblkAKCObedClassInfo)
# ---------------------------------------------------------------------------
# This is the master class catalog spanning Obedience and Rally. Each row
# describes a class definition: its scoring scale, jump requirements,
# typical dogs-per-hour throughput, and AKC-internal class code. We
# preserve the full set including "Transfer to X" pseudo-rows because they
# encode which classes can transfer to which targets (a domain rule).

def clean_canonical_classes():
    src_rows = read_access_csv(SRC / "raw_tblkAKCObedClassInfo.csv")
    out_rows = []
    for r in src_rows:
        out_rows.append({
            "legacy_id": int(r["ClassID"]),
            "name": strip_trailing_whitespace(r["Class"]),
            "sport": strip_trailing_whitespace(r["Event"]),
            "class_type": strip_trailing_whitespace(r["ClassType"]),
            "akc_class_code": int(r["ClassCode"]) if r.get("ClassCode") else None,
            "moveup_target_legacy_id": int(r["MoveUp"]) if r.get("MoveUp") else None,
            "is_sanctioned": bool_from_access(r.get("Sanctioned")),
            "actual_class_name": strip_trailing_whitespace(r["ActualClass"]),
            "is_displayed": bool_from_access(r.get("Show")),
            "has_jump": bool_from_access(r.get("HasJump")),
            "has_multiple_entries": bool_from_access(r.get("HasMultEntries")),
            "max_total_score": int(r["TotalScore"]) if r.get("TotalScore") else None,
            "default_dogs_per_hour": int(r["DogsPerHour"]) if r.get("DogsPerHour") else None,
            "has_broad_jump": bool_from_access(r.get("BroadJump")),
            "has_walkthrough": bool_from_access(r.get("Walkthrough")),
        })
    fieldnames = [
        "legacy_id", "name", "sport", "class_type", "akc_class_code",
        "moveup_target_legacy_id", "is_sanctioned", "actual_class_name",
        "is_displayed", "has_jump", "has_multiple_entries",
        "max_total_score", "default_dogs_per_hour", "has_broad_jump",
        "has_walkthrough",
    ]
    write_csv(OUT / "canonical_classes.csv", fieldnames, out_rows)
    print(f"  canonical_classes.csv: {len(out_rows)} rows")


# ---------------------------------------------------------------------------
# rally_rach_points (from tblAKCRallyRACHPoints)
# ---------------------------------------------------------------------------
# Maps a Rally Master/Excellent B/Advanced B qualifying score to the
# RACH (Rally Champion) points it earns. The table has TWO columns of
# point values: the original (RACHPoints) and the post-2019-02-01 revised
# values (RACHPoints20190201). Both are preserved so historical data can
# be calculated correctly using the rules in effect at trial time.

def clean_rally_rach_points():
    """RACH (Rally Champion) points per qualifying score.

    Deborah's answer (Q3.2, 2026-04-20): QTrial only needs to track the CURRENT
    rules. Historical scoring is AKC's responsibility and trial secretaries
    take handlers' word for prior titles earned. So we drop the pre-2019-02-01
    values and emit only the current RACHPoints20190201 column.
    """
    src_rows = read_access_csv(SRC / "raw_tblAKCRallyRACHPoints.csv")
    out_rows = []
    for r in src_rows:
        out_rows.append({
            "legacy_id": int(r["RACHPointsID"]),
            "score": float(r["Score"]),
            "rach_points": int(r["RACHPoints20190201"]),
        })
    fieldnames = ["legacy_id", "score", "rach_points"]
    write_csv(OUT / "rally_rach_points.csv", fieldnames, out_rows)
    print(f"  rally_rach_points.csv: {len(out_rows)} rows")


# ---------------------------------------------------------------------------
# om_points (from tblAKCObedienceOMPoints)
# ---------------------------------------------------------------------------
# Maps an Open B / Utility B qualifying score (190-200) to the OM
# (Obedience Master) points it earns.

def clean_om_points():
    src_rows = read_access_csv(SRC / "raw_tblAKCObedienceOMPoints.csv")
    out_rows = []
    for r in src_rows:
        # Source column is OtchPointsID (not OMPointsID) - typo preserved
        out_rows.append({
            "legacy_id": int(r["OtchPointsID"]),
            "score": float(r["Score"]),
            "om_points": int(r["OMPoints"]),
        })
    fieldnames = ["legacy_id", "score", "om_points"]
    write_csv(OUT / "om_points.csv", fieldnames, out_rows)
    print(f"  om_points.csv:       {len(out_rows)} rows")


# ---------------------------------------------------------------------------
# otch_points (from tblAKCObedienceOtchPoints)
# ---------------------------------------------------------------------------
# OTCH (Obedience Trial Champion) points awarded for Open B and Utility B
# placements based on the number of dogs in the class.
#
# Deborah's answer (Q3.1, 2026-04-20): QTrial only needs to track the CURRENT
# rules. Historical scoring is AKC's responsibility. The pre-Aug-2019 variant
# (tblAKCObedienceOtchPoints20190801) is NOT imported.

def clean_otch_points():
    current_rows = read_access_csv(SRC / "raw_tblAKCObedienceOtchPoints.csv")
    out_rows = []
    for r in current_rows:
        out_rows.append({
            "legacy_id": int(r["OtchPointsID"]),
            "class_name": strip_trailing_whitespace(r["Class"]),
            "entries_min": int(r["Min"]),
            "entries_max": int(r["Max"]),
            "first_place_points": int(r["First"]),
            "second_place_points": int(r["Second"]),
            "third_place_points": int(r["Third"]),
            "fourth_place_points": int(r["Fourth"]),
        })
    fieldnames = [
        "legacy_id", "class_name", "entries_min", "entries_max",
        "first_place_points", "second_place_points",
        "third_place_points", "fourth_place_points",
    ]
    write_csv(OUT / "otch_points.csv", fieldnames, out_rows)
    print(f"  otch_points.csv:     {len(out_rows)} rows (current AKC rules)")


# ---------------------------------------------------------------------------
# obedience_exercises (from tblAKCObedienceExercises)
# ---------------------------------------------------------------------------
# The catalog of individual obedience exercises (Heel on Leash, Recall, etc.)

def clean_obedience_exercises():
    src_rows = read_access_csv(SRC / "raw_tblAKCObedienceExercises.csv")
    if not src_rows:
        print("  obedience_exercises.csv: SKIPPED (empty source)")
        return
    out_rows = []
    # Use the actual column names from the source - we'll discover them
    fieldnames_src = list(src_rows[0].keys())
    # Generic pass-through with snake_case column renames
    for r in src_rows:
        out_row = {}
        for k, v in r.items():
            out_row[k] = strip_trailing_whitespace(v) if isinstance(v, str) else v
        out_rows.append(out_row)
    write_csv(OUT / "obedience_exercises.csv", fieldnames_src, out_rows)
    print(f"  obedience_exercises.csv: {len(out_rows)} rows ({len(fieldnames_src)} cols)")


# ---------------------------------------------------------------------------
# obedience_class_exercises (from tblAKCObedienceClassExercises)
# ---------------------------------------------------------------------------
# Junction table mapping classes to the exercises performed in them.
# Has a 20180501 archive sibling (pre-May-2018 exercise definitions).

def clean_obedience_class_exercises():
    current_rows = read_access_csv(SRC / "raw_tblAKCObedienceClassExercises.csv")
    if not current_rows:
        print("  obedience_class_exercises.csv: SKIPPED (empty source)")
        return
    fieldnames_src = list(current_rows[0].keys())
    out_rows = []
    for r in current_rows:
        out_row = {}
        for k, v in r.items():
            out_row[k] = strip_trailing_whitespace(v) if isinstance(v, str) else v
        out_rows.append(out_row)
    write_csv(OUT / "obedience_class_exercises.csv", fieldnames_src, out_rows)
    print(f"  obedience_class_exercises.csv: {len(out_rows)} rows ({len(fieldnames_src)} cols)")


# ---------------------------------------------------------------------------
# jumps (from tblAKCObedienceJumps)
# ---------------------------------------------------------------------------
# Jump height metadata for obedience classes.

def clean_jumps():
    src_rows = read_access_csv(SRC / "raw_tblAKCObedienceJumps.csv")
    if not src_rows:
        print("  jumps.csv: SKIPPED (empty source)")
        return
    fieldnames_src = list(src_rows[0].keys())
    out_rows = []
    for r in src_rows:
        out_row = {}
        for k, v in r.items():
            out_row[k] = strip_trailing_whitespace(v) if isinstance(v, str) else v
        out_rows.append(out_row)
    write_csv(OUT / "jumps.csv", fieldnames_src, out_rows)
    print(f"  jumps.csv:           {len(out_rows)} rows ({len(fieldnames_src)} cols)")


# ---------------------------------------------------------------------------
# akc_xml_class_codes (from tblAKCxmlClassNames) - POST-MVP ONLY
# ---------------------------------------------------------------------------
# AKC's official XML class identifier codes used in electronic results submission.
# Covers Agility only (Standard, JWW, FAST).
#
# Deborah's answer (Q4, 2026-04-20): AKC Obedience and Rally submission is
# PDF-based, not XML-based. The AKC Report of Rally Trial (Form JOVRY8) plus
# the marked catalog plus the judges books are emailed to rallyresults@akc.org.
# There is no XML submission for MVP sports, so this file is routed to the
# post_mvp/ subdirectory and will become relevant when QTrial adds Agility.

def clean_akc_xml_class_codes():
    src_rows = read_access_csv(SRC / "raw_tblAKCxmlClassNames.csv")
    out_rows = []
    for r in src_rows:
        out_rows.append({
            "sport": strip_trailing_whitespace(r["Event"]),
            "level": strip_trailing_whitespace(r["Level"]),
            "class_name": strip_trailing_whitespace(r["Class"]),
            "akc_xml_code": strip_trailing_whitespace(r["AKCPrimaryClass"]),
        })
    fieldnames = ["sport", "level", "class_name", "akc_xml_code"]
    post_mvp_dir = OUT / "post_mvp"
    post_mvp_dir.mkdir(exist_ok=True)
    write_csv(post_mvp_dir / "akc_xml_class_codes.csv", fieldnames, out_rows)
    print(f"  post_mvp/akc_xml_class_codes.csv: {len(out_rows)} rows (Agility only; post-MVP)")


# ---------------------------------------------------------------------------
# akc_xml_jump_heights (from tblAKCxmlJumpHeights) - POST-MVP ONLY
# ---------------------------------------------------------------------------
# Same reasoning as akc_xml_class_codes. Agility-only, post-MVP.

def clean_akc_xml_jump_heights():
    src_rows = read_access_csv(SRC / "raw_tblAKCxmlJumpHeights.csv")
    if not src_rows:
        print("  post_mvp/akc_xml_jump_heights.csv: SKIPPED (empty source)")
        return
    fieldnames_src = list(src_rows[0].keys())
    out_rows = []
    for r in src_rows:
        out_row = {}
        for k, v in r.items():
            out_row[k] = strip_trailing_whitespace(v) if isinstance(v, str) else v
        out_rows.append(out_row)
    post_mvp_dir = OUT / "post_mvp"
    post_mvp_dir.mkdir(exist_ok=True)
    write_csv(post_mvp_dir / "akc_xml_jump_heights.csv", fieldnames_src, out_rows)
    print(f"  post_mvp/akc_xml_jump_heights.csv: {len(out_rows)} rows ({len(fieldnames_src)} cols; post-MVP)")


# ---------------------------------------------------------------------------
# non_akc_title_suffixes (from tblAKCTitlesSuffixNonAKC)
# ---------------------------------------------------------------------------
# Suffix titles from non-AKC issuing organizations (breed clubs, working
# dog organizations like Schutzhund, etc.). DATA_MODEL.md does not yet
# have a dedicated table for these - PROPOSAL: add a `source_organization`
# column to title_suffixes and merge these in, OR keep as separate table.

def clean_non_akc_title_suffixes():
    src_rows = read_access_csv(SRC / "raw_tblAKCTitlesSuffixNonAKC.csv")
    out_rows = []
    for r in src_rows:
        out_rows.append({
            "legacy_id": int(r["AKCSuffixID"]),
            "code": strip_trailing_whitespace(r["AKCSuffix"]),
            "sport_scope_code": strip_trailing_whitespace(r.get("Type", "")),
            "display_order": int(r["SuffixOrder"]) if r.get("SuffixOrder") else None,
            "source_organization": strip_trailing_whitespace(r.get("Source", "")),
            "long_name": strip_trailing_whitespace(r.get("Name", "")),
            "allows_multiple": bool_from_access(r.get("Multiple")),
        })
    fieldnames = [
        "legacy_id", "code", "sport_scope_code", "display_order",
        "source_organization", "long_name", "allows_multiple",
    ]
    write_csv(OUT / "non_akc_title_suffixes.csv", fieldnames, out_rows)
    print(f"  non_akc_title_suffixes.csv: {len(out_rows)} rows")


# ---------------------------------------------------------------------------
# non_akc_title_suffix_breed_restrictions (from tblAKCTitlesSuffixNonAKCBreed)
# ---------------------------------------------------------------------------
# Junction table: which non-AKC titles are restricted to which breeds?
# (e.g., "Carting Started" is offered by the American Rottweiler Club
# specifically, primarily for Rottweilers.)

def clean_non_akc_title_breed_restrictions():
    src_rows = read_access_csv(SRC / "raw_tblAKCTitlesSuffixNonAKCBreed.csv")
    if not src_rows:
        print("  non_akc_title_suffix_breed_restrictions.csv: SKIPPED (empty source)")
        return
    fieldnames_src = list(src_rows[0].keys())
    out_rows = []
    for r in src_rows:
        out_row = {}
        for k, v in r.items():
            out_row[k] = strip_trailing_whitespace(v) if isinstance(v, str) else v
        out_rows.append(out_row)
    write_csv(OUT / "non_akc_title_suffix_breed_restrictions.csv", fieldnames_src, out_rows)
    print(f"  non_akc_title_suffix_breed_restrictions.csv: {len(out_rows)} rows ({len(fieldnames_src)} cols)")


# ---------------------------------------------------------------------------
# trial_time_calculations (from tblTrialTimeCalculation + MVP additions)
# ---------------------------------------------------------------------------
# Configurable defaults for estimating trial time per sport.
#
# Deborah's answer (Q5.1, 2026-04-20): AKC does not publish official time
# defaults. Rally and Obedience are scored with per-dog time tracking (for
# tie-breaking), but estimation rule-of-thumb is ~3 minutes per dog.
# Her actual Nov 2025 schedule showed class-dependent pacing (Rally Choice
# ~4.3 min/dog, Rally Master ~3.5, Rally Excellent B ~3.1), averaging close
# to 3.0. Per-event overrides will let secretaries deviate from the default.

# Additional defaults for sports missing from Deborah's source table.
# These are appended after the Agility defaults loaded from her database.
MVP_TRIAL_TIME_ADDITIONS = [
    # (sport_or_event, minutes_per_dog, class_change_seconds, event_change_seconds)
    ("Obedience", 3.0, 30, 45),
    ("Rally", 3.0, 30, 45),
]


def clean_trial_time_calculations():
    src_rows = read_access_csv(SRC / "raw_tblTrialTimeCalculation.csv")
    if not src_rows:
        print("  trial_time_calculations.csv: SKIPPED (empty source)")
        return
    out_rows = []
    for r in src_rows:
        out_rows.append({
            "legacy_id": int(r["TrialTimeID"]),
            "sport_or_event": strip_trailing_whitespace(r["Event"]),
            "minutes_per_dog": float(r["RunTime"]),
            "class_change_seconds": int(r["ClassChangeTime"]),
            "event_change_seconds": int(r["EventChangeTime"]),
        })

    # Append MVP additions for sports missing from Deborah's source
    existing_sports = {r["sport_or_event"] for r in out_rows}
    next_id = max((r["legacy_id"] for r in out_rows), default=0) + 1
    for sport, mpd, class_sec, event_sec in MVP_TRIAL_TIME_ADDITIONS:
        if sport not in existing_sports:
            out_rows.append({
                "legacy_id": next_id,
                "sport_or_event": sport,
                "minutes_per_dog": mpd,
                "class_change_seconds": class_sec,
                "event_change_seconds": event_sec,
            })
            next_id += 1

    fieldnames = [
        "legacy_id", "sport_or_event", "minutes_per_dog",
        "class_change_seconds", "event_change_seconds",
    ]
    write_csv(OUT / "trial_time_calculations.csv", fieldnames, out_rows)
    print(f"  trial_time_calculations.csv: {len(out_rows)} rows")


# ===========================================================================
# ===========================================================================
# AKC AUTHORITATIVE TITLE CATALOG
# ===========================================================================
#
# Transcribed from https://www.akc.org/sports/titles-and-abbreviations/
# Fetched 2026-04-20.
#
# Decision (2026-04-20): When current AKC published structure conflicts with
# Deborah's database, AKC wins. Lab Tested Databases (ObedienceData.accde) is
# substantially behind AKC on post-2018 additions (scent work, trick dog,
# therapy dog, virtual titles, FIT Dog, Fetch, etc.). These embedded lists are
# the ground truth for the title catalog.
#
# Regeneration workflow: when AKC updates their titles page, edit the lists
# below by hand. The legacy_id mapping happens automatically in the cleaning
# functions by looking up codes in Deborah's raw extract; new codes get IDs
# in the 1000+ range.
#
# Bundled herding codes (HIAdsc etc. on AKC's page) are expanded into
# individual duck/sheep/cattle rows because they appear as distinct codes
# in registered dog names.

# Format: (code, long_name, sport_scope_code, sport_scope_description)

AKC_PREFIX_TITLES = [
    ("AFC", "Amateur Field Trial Champion", "F", "Field Trial"),
    ("AGCH", "Agility Grand Champion", "A", "Agility"),
    ("CCH", "Coonhound Bench Show Champion", "M", "Coonhound"),
    ("CFC", "Coonhound Field Champion", "M", "Coonhound"),
    ("CGCH", "Coonhound Grand Champion", "M", "Coonhound"),
    ("CGF", "Coonhound Grand Field Champion", "M", "Coonhound"),
    ("CGN", "Coonhound Grand Nite Champion", "M", "Coonhound"),
    ("CGW", "Coonhound Grand Water Race Champion", "M", "Coonhound"),
    ("CH", "Champion", "N", "Conformation"),
    ("CNC", "Coonhound Nite Champion", "M", "Coonhound"),
    ("CSG", "Coonhound Supreme Grand Champion", "M", "Coonhound"),
    ("CSGF", "Coonhound Supreme Grand Field Champion", "M", "Coonhound"),
    ("CSGN", "Coonhound Supreme Grand Nite Champion", "M", "Coonhound"),
    ("CSGW", "Coonhound Supreme Grand Water Race Champion", "M", "Coonhound"),
    ("CT", "Champion Tracker", "T", "Tracking"),
    ("CWC", "Coonhound Water Race Champion", "M", "Coonhound"),
    ("CWSG", "Coonhound World Show Champion", "M", "Coonhound"),
    ("DC", "Dual Champion", "V", "Multi-sport (Conformation + Field/Herding)"),
    ("FC", "Field Champion", "F", "Field Trial"),
    ("GCH", "Grand Champion", "N", "Conformation"),
    ("GCHB", "Grand Champion Bronze", "N", "Conformation"),
    ("GCHG", "Grand Champion Gold", "N", "Conformation"),
    ("GCHP", "Grand Champion Platinum", "N", "Conformation"),
    ("GCHS", "Grand Champion Silver", "N", "Conformation"),
    ("GDSC", "Gun Dog Stake Champion", "F", "Field Trial"),
    ("GAFC", "Grand Amateur Field Champion", "F", "Field Trial"),
    ("GFC", "Grand Field Champion", "F", "Field Trial"),
    ("HC", "Herding Champion", "H", "Herding"),
    ("MACH", "Master Agility Champion", "A", "Agility"),
    ("NAC", "National Agility Champion", "A", "Agility"),
    ("NAFC", "National Amateur Field Champion", "F", "Field Trial"),
    ("NAGDC", "National Amateur Gundog Champion", "F", "Field Trial"),
    ("NFC", "National Field Champion", "F", "Field Trial"),
    ("NGDC", "National Gundog Champion", "F", "Field Trial"),
    ("NOC", "National Obedience Champion", "O", "Obedience"),
    ("OTCH", "Obedience Trial Champion", "O", "Obedience"),
    ("PACH", "Preferred Agility Champion", "A", "Agility"),
    ("PNAC", "Preferred National Agility Champion", "A", "Agility"),
    ("POC", "Preferred Obedience Champion", "O", "Obedience"),
    ("RACH", "Rally Champion", "R", "Rally"),
    ("RAFC", "Regional Amateur Field Champion", "F", "Field Trial"),
    ("RFC", "Regional Field Champion", "F", "Field Trial"),
    ("RGDSC", "Retrieving Gun Dog Stake Champion", "F", "Field Trial"),
    ("RNC", "Rally National Champion", "R", "Rally"),
    ("RWAFC", "Regional Walking Amateur Field Champion", "F", "Field Trial"),
    ("RWFC", "Regional Walking Field Champion", "F", "Field Trial"),
    ("TC", "Triple Champion", "V", "Multi-sport (DC + OTCH/CT/MACH/PACH)"),
    ("VCCH", "Versatile Companion Champion", "V", "Multi-sport (OTCH + MACH/PACH + CT)"),
    ("WNC", "World Nite Champion", "M", "Coonhound"),
]

AKC_SUFFIX_TITLES = [
    # Agility Course Tests
    ("ACT1", "Agility Course Test 1", "A", "Agility"),
    ("ACT1J", "Agility Course Test 1 Jumpers", "J", "Agility Jumpers With Weaves"),
    ("ACT2", "Agility Course Test 2", "A", "Agility"),
    ("ACT2J", "Agility Course Test 2 Jumpers", "J", "Agility Jumpers With Weaves"),
    # Agility suffix titles (regular, preferred, and Premier)
    ("AJP", "Excellent Agility Jumpers With Weaves A Preferred", "Q", "Preferred Agility Jumpers"),
    ("AX", "Agility Excellent", "A", "Agility"),
    ("AXJ", "Excellent Agility Jumper", "J", "Agility Jumpers With Weaves"),
    ("AXP", "Agility Excellent A Preferred", "P", "Preferred Agility Standard"),
    # Temperament Test
    ("ATT", "AKC Temperament Test", "TT", "Temperament Test"),
    # Fast CAT
    ("BCAT", "BCAT - 150 points in Fast CAT events", "FCAT", "Fast CAT"),
    # Obedience core
    ("BN", "Beginner Novice", "O", "Obedience"),
    ("BN-V", "Virtual Beginner Novice", "G", "Virtual Obedience"),
    # Coursing Ability
    ("CA", "Coursing Ability", "C", "Coursing"),
    ("CAA", "Coursing Ability Advanced", "C", "Coursing"),
    ("CAX", "Coursing Ability Excellent", "C", "Coursing"),
    ("CAX2", "Coursing Ability Excellent 2", "C", "Coursing"),
    # Obedience core continued
    ("CD", "Companion Dog", "O", "Obedience"),
    ("CD-V", "Virtual Companion Dog", "G", "Virtual Obedience"),
    ("CDX", "Companion Dog Excellent", "O", "Obedience"),
    # Canine Good Citizen
    ("CGC", "Canine Good Citizen", "X", "Companion"),
    ("CGCA", "AKC Community Canine", "X", "Companion"),
    ("CGCU", "Canine Good Citizen Urban", "X", "Companion"),
    # Certificate of Merit
    ("CM", "Certificate of Merit", "N", "Conformation"),
    # Fast CAT continued
    ("DCAT", "DCAT - 500 points in Fast CAT events", "FCAT", "Fast CAT"),
    # Earthdog
    ("EE", "Endurance Earthdog", "E", "Earthdog"),
    ("FCAT", "FCAT - 1000 points in Fast CAT events", "FCAT", "Fast CAT"),
    # Farm Dog Certified
    ("FDC", "Farm Dog Certified", "X", "Companion"),
    # FIT Dog
    ("FITB", "Bronze AKC FIT DOG", "I", "FIT Dog"),
    ("FITG", "Gold AKC FIT DOG", "I", "FIT Dog"),
    ("FITS", "Silver AKC FIT DOG", "I", "FIT Dog"),
    # Fetch
    ("FTA", "Fetch Advanced", "B", "Fetch"),
    ("FTC", "FAST Century", "Y", "Agility FAST"),
    ("FTCP", "FAST Century Preferred", "Z", "Preferred Agility FAST"),
    ("FTI", "Fetch Intermediate", "B", "Fetch"),
    ("FTN", "Fetch Novice", "B", "Fetch"),
    ("FTR", "Fetch Retriever", "B", "Fetch"),
    # Obedience Graduate classes
    ("GN", "Graduate Novice", "O", "Obedience"),
    ("GO", "Graduate Open", "O", "Obedience"),
    # Herding intermediate
    ("HI", "Herding Intermediate", "H", "Herding"),
    ("HIAd", "Herding Intermediate Course A - Ducks", "H", "Herding"),
    ("HIAs", "Herding Intermediate Course A - Sheep", "H", "Herding"),
    ("HIAc", "Herding Intermediate Course A - Cattle", "H", "Herding"),
    ("HIBd", "Herding Intermediate Course B - Ducks", "H", "Herding"),
    ("HIBs", "Herding Intermediate Course B - Sheep", "H", "Herding"),
    ("HIBc", "Herding Intermediate Course B - Cattle", "H", "Herding"),
    ("HICs", "Herding Intermediate Course C - Sheep", "H", "Herding"),
    # Herding started
    ("HS", "Herding Started", "H", "Herding"),
    ("HSAd", "Herding Started Course A - Ducks", "H", "Herding"),
    ("HSAs", "Herding Started Course A - Sheep", "H", "Herding"),
    ("HSAc", "Herding Started Course A - Cattle", "H", "Herding"),
    ("HSBd", "Herding Started Course B - Ducks", "H", "Herding"),
    ("HSBs", "Herding Started Course B - Sheep", "H", "Herding"),
    ("HSBc", "Herding Started Course B - Cattle", "H", "Herding"),
    ("HSCs", "Herding Started Course C - Sheep", "H", "Herding"),
    # Herding tested / advanced
    ("HT", "Herding Tested", "H", "Herding"),
    ("HX", "Herding Excellent", "H", "Herding"),
    ("HXAd", "Herding Advanced Course A - Ducks", "H", "Herding"),
    ("HXAs", "Herding Advanced Course A - Sheep", "H", "Herding"),
    ("HXAc", "Herding Advanced Course A - Cattle", "H", "Herding"),
    ("HXBd", "Herding Advanced Course B - Ducks", "H", "Herding"),
    ("HXBs", "Herding Advanced Course B - Sheep", "H", "Herding"),
    ("HXBc", "Herding Advanced Course B - Cattle", "H", "Herding"),
    ("HXCs", "Herding Advanced Course C - Sheep", "H", "Herding"),
    # Lure Coursing
    ("JC", "Junior Courser", "L", "Lure Coursing"),
    # Earthdog
    ("JE", "Junior Earthdog", "E", "Earthdog"),
    # Hunt Test
    ("JH", "Junior Hunter", "F", "Hunt Test"),
    ("JHA", "Junior Hunter Advanced", "F", "Hunt Test"),
    ("JHR", "Junior Hunter Retriever", "F", "Hunt Test"),
    ("JHU", "Junior Hunter Upland", "F", "Hunt Test"),
    # Lure Coursing continued
    ("LCX", "Lure Courser Excellent", "L", "Lure Coursing"),
    ("MC", "Master Courser", "L", "Lure Coursing"),
    # Earthdog
    ("ME", "Master Earthdog", "E", "Earthdog"),
    # Agility FAST Master variants
    ("MFB", "Master Bronze FAST", "Y", "Agility FAST"),
    ("MFC", "Master Century FAST", "Y", "Agility FAST"),
    ("MFG", "Master Gold FAST", "Y", "Agility FAST"),
    ("MFP", "Master Excellent FAST Preferred", "Z", "Preferred Agility FAST"),
    ("MFPB", "Master Bronze FAST Preferred", "Z", "Preferred Agility FAST"),
    ("MFPC", "Master Century FAST Preferred", "Z", "Preferred Agility FAST"),
    ("MFPG", "Master Gold FAST Preferred", "Z", "Preferred Agility FAST"),
    ("MFPS", "Master Silver FAST Preferred", "Z", "Preferred Agility FAST"),
    ("MFS", "Master Silver FAST", "Y", "Agility FAST"),
    # Hunt Test Master
    ("MH", "Master Hunter", "F", "Hunt Test"),
    ("MHA", "Master Hunter Advanced", "F", "Hunt Test"),
    ("MHR", "Master Hunter Retriever", "F", "Hunt Test"),
    ("MHU", "Master Hunter Upland", "F", "Hunt Test"),
    # Agility Master Jumpers variants
    ("MJB", "Master Bronze Jumpers With Weaves", "J", "Agility Jumpers With Weaves"),
    ("MJC", "Master Century Jumpers With Weaves", "J", "Agility Jumpers With Weaves"),
    ("MJG", "Master Gold Jumpers With Weaves", "J", "Agility Jumpers With Weaves"),
    ("MJP", "Master Excellent Jumpers With Weaves Preferred", "Q", "Preferred Agility Jumpers"),
    ("MJPB", "Master Bronze Jumpers With Weaves Preferred", "Q", "Preferred Agility Jumpers"),
    ("MJPC", "Master Century Jumpers With Weaves Preferred", "Q", "Preferred Agility Jumpers"),
    ("MJPG", "Master Gold Jumpers With Weaves Preferred", "Q", "Preferred Agility Jumpers"),
    ("MJPS", "Master Silver Jumpers With Weaves Preferred", "Q", "Preferred Agility Jumpers"),
    ("MJS", "Master Silver Jumpers With Weaves", "J", "Agility Jumpers With Weaves"),
    # Hunt Test National
    ("MNH", "Master National Hunter", "F", "Hunt Test"),
    # Agility Master Standard variants
    ("MX", "Master Agility Excellent", "A", "Agility"),
    ("MXB", "Master Bronze Agility", "A", "Agility"),
    ("MXC", "Master Century Agility", "A", "Agility"),
    ("MXF", "Master Excellent FAST", "Y", "Agility FAST"),
    ("MXG", "Master Gold Agility", "A", "Agility"),
    ("MXJ", "Master Excellent Jumpers With Weaves", "J", "Agility Jumpers With Weaves"),
    ("MXP", "Master Agility Excellent Preferred", "P", "Preferred Agility Standard"),
    ("MXPB", "Master Bronze Agility Preferred", "P", "Preferred Agility Standard"),
    ("MXPC", "Master Century Agility Preferred", "P", "Preferred Agility Standard"),
    ("MXPG", "Master Gold Agility Preferred", "P", "Preferred Agility Standard"),
    ("MXPS", "Master Silver Agility Preferred", "P", "Preferred Agility Standard"),
    ("MXS", "Master Silver Agility", "A", "Agility"),
    # Agility Novice/Open levels
    ("NA", "Novice Agility", "A", "Agility"),
    ("NAJ", "Novice Agility Jumper", "J", "Agility Jumpers With Weaves"),
    ("NAP", "Novice Agility Preferred", "P", "Preferred Agility Standard"),
    # Earthdog Novice
    ("NE", "Novice Earthdog", "E", "Earthdog"),
    # Agility Novice continued
    ("NF", "Novice FAST", "Y", "Agility FAST"),
    ("NFP", "Novice FAST Preferred", "Z", "Preferred Agility FAST"),
    ("NJP", "Novice Jumpers With Weaves Preferred", "Q", "Preferred Agility Jumpers"),
    # Agility Open
    ("OA", "Open Agility", "A", "Agility"),
    ("OAJ", "Open Agility Jumper", "J", "Agility Jumpers With Weaves"),
    ("OAP", "Open Agility Preferred", "P", "Preferred Agility Standard"),
    ("OF", "Open FAST", "Y", "Agility FAST"),
    ("OFP", "Open FAST Preferred", "Z", "Preferred Agility FAST"),
    # Obedience advanced
    ("OGM", "Obedience Grand Master", "O", "Obedience"),
    # Agility Open continued
    ("OJP", "Open Jumpers With Weaves Preferred", "Q", "Preferred Agility Jumpers"),
    # Obedience Master
    ("OM", "Obedience Master", "O", "Obedience"),
    # Premier Agility
    ("PAD", "Premier Agility Dog", "A", "Agility Premier"),
    ("PADP", "Premier Agility Dog Preferred", "P", "Preferred Agility Premier"),
    ("PAX", "Preferred Agility Excellent", "P", "Preferred Agility Standard"),
    # Preferred Obedience
    ("PCD", "Preferred Companion Dog", "O", "Obedience"),
    ("PCDX", "Preferred Companion Dog Excellent", "O", "Obedience"),
    # Premier Agility Dog tiers
    ("PDB", "Premier Agility Dog Bronze", "A", "Agility Premier"),
    ("PDBP", "Premier Agility Dog Bronze Preferred", "P", "Preferred Agility Premier"),
    ("PDC", "Premier Agility Dog Century", "A", "Agility Premier"),
    ("PDCP", "Premier Agility Dog Century Preferred", "P", "Preferred Agility Premier"),
    ("PDG", "Premier Agility Dog Gold", "A", "Agility Premier"),
    ("PDGP", "Premier Agility Dog Gold Preferred", "P", "Preferred Agility Premier"),
    ("PDS", "Premier Agility Dog Silver", "A", "Agility Premier"),
    ("PDSP", "Premier Agility Dog Silver Preferred", "P", "Preferred Agility Premier"),
    # Premier Jumpers Dog tiers
    ("PJB", "Premier Jumpers Dog Bronze", "J", "Agility Jumpers Premier"),
    ("PJBP", "Premier Jumpers Dog Bronze Preferred", "Q", "Preferred Jumpers Premier"),
    ("PJC", "Premier Jumpers Dog Century", "J", "Agility Jumpers Premier"),
    ("PJCP", "Premier Jumpers Dog Century Preferred", "Q", "Preferred Jumpers Premier"),
    ("PJD", "Premier Jumpers Dog", "J", "Agility Jumpers Premier"),
    ("PJDP", "Premier Jumpers Dog Preferred", "Q", "Preferred Jumpers Premier"),
    ("PJG", "Premier Jumpers Dog Gold", "J", "Agility Jumpers Premier"),
    ("PJGP", "Premier Jumpers Dog Gold Preferred", "Q", "Preferred Jumpers Premier"),
    ("PJS", "Premier Jumpers Dog Silver", "J", "Agility Jumpers Premier"),
    ("PJSP", "Premier Jumpers Dog Silver Preferred", "Q", "Preferred Jumpers Premier"),
    # Herding Pre-Trial
    ("PT", "Pre-Trial Tested", "H", "Herding"),
    # Preferred Utility Dog variants
    ("PUDX", "Preferred Utility Dog Excellent", "O", "Obedience"),
    ("PUTD", "Preferred Utility Dog", "O", "Obedience"),
    # Rally
    ("RA", "Rally Advanced", "R", "Rally"),
    ("RAE", "Rally Advanced Excellent", "R", "Rally"),
    ("RC", "Rally Choice", "R", "Rally"),
    ("RE", "Rally Excellent", "R", "Rally"),
    ("RI", "Rally Intermediate", "R", "Rally"),
    ("RM", "Rally Master", "R", "Rally"),
    ("RN", "Rally Novice", "R", "Rally"),
    # Scent Work - Buried
    ("SBA", "Scent Work Buried Advanced", "W", "Scent Work"),
    ("SBAE", "Scent Work Buried Advanced Elite", "W", "Scent Work"),
    ("SBE", "Scent Work Buried Excellent", "W", "Scent Work"),
    ("SBEE", "Scent Work Buried Excellent Elite", "W", "Scent Work"),
    ("SBM", "Scent Work Buried Master", "W", "Scent Work"),
    ("SBME", "Scent Work Buried Master Elite", "W", "Scent Work"),
    ("SBN", "Scent Work Buried Novice", "W", "Scent Work"),
    ("SBNE", "Scent Work Buried Novice Elite", "W", "Scent Work"),
    # Lure Coursing Senior
    ("SC", "Senior Courser", "L", "Lure Coursing"),
    # Scent Work - Container
    ("SCA", "Scent Work Container Advanced", "W", "Scent Work"),
    ("SCAE", "Scent Work Container Advanced Elite", "W", "Scent Work"),
    ("SCE", "Scent Work Container Excellent", "W", "Scent Work"),
    ("SCEE", "Scent Work Container Excellent Elite", "W", "Scent Work"),
    ("SCM", "Scent Work Container Master", "W", "Scent Work"),
    ("SCME", "Scent Work Container Master Elite", "W", "Scent Work"),
    ("SCN", "Scent Work Container Novice", "W", "Scent Work"),
    ("SCNE", "Scent Work Container Novice Elite", "W", "Scent Work"),
    # Earthdog Senior
    ("SE", "Senior Earthdog", "E", "Earthdog"),
    # Scent Work - Exterior
    ("SEA", "Scent Work Exterior Advanced", "W", "Scent Work"),
    ("SEAE", "Scent Work Exterior Advanced Elite", "W", "Scent Work"),
    ("SEE", "Scent Work Exterior Excellent", "W", "Scent Work"),
    ("SEEE", "Scent Work Exterior Excellent Elite", "W", "Scent Work"),
    ("SEM", "Scent Work Exterior Master", "W", "Scent Work"),
    ("SEME", "Scent Work Exterior Master Elite", "W", "Scent Work"),
    ("SEN", "Scent Work Exterior Novice", "W", "Scent Work"),
    ("SENE", "Scent Work Exterior Novice Elite", "W", "Scent Work"),
    # Hunt Test Senior
    ("SH", "Senior Hunter", "F", "Hunt Test"),
    ("SHA", "Senior Hunter Advanced", "F", "Hunt Test"),
    # Scent Work - Handler Discrimination
    ("SHDA", "Scent Work Handler Discrimination Advanced", "W", "Scent Work"),
    ("SHDAE", "Scent Work Handler Discrimination Advanced Elite", "W", "Scent Work"),
    ("SHDE", "Scent Work Handler Discrimination Excellent", "W", "Scent Work"),
    ("SHDEE", "Scent Work Handler Discrimination Excellent Elite", "W", "Scent Work"),
    ("SHDM", "Scent Work Handler Discrimination Master", "W", "Scent Work"),
    ("SHDME", "Scent Work Handler Discrimination Master Elite", "W", "Scent Work"),
    ("SHDN", "Scent Work Handler Discrimination Novice", "W", "Scent Work"),
    ("SHDNE", "Scent Work Handler Discrimination Novice Elite", "W", "Scent Work"),
    # Hunt Test Senior continued
    ("SHR", "Senior Hunter Retriever", "F", "Hunt Test"),
    ("SHU", "Senior Hunter Upland", "F", "Hunt Test"),
    # Scent Work - Interior
    ("SIA", "Scent Work Interior Advanced", "W", "Scent Work"),
    ("SIAE", "Scent Work Interior Advanced Elite", "W", "Scent Work"),
    ("SIE", "Scent Work Interior Excellent", "W", "Scent Work"),
    ("SIEE", "Scent Work Interior Excellent Elite", "W", "Scent Work"),
    ("SIM", "Scent Work Interior Master", "W", "Scent Work"),
    ("SIME", "Scent Work Interior Master Elite", "W", "Scent Work"),
    ("SIN", "Scent Work Interior Novice", "W", "Scent Work"),
    ("SINE", "Scent Work Interior Novice Elite", "W", "Scent Work"),
    # STAR Puppy (AKC uses STR per current Titles page; not SP)
    ("STR", "STAR Puppy", "X", "Companion"),
    # Scent Work combined titles
    ("SWA", "Scent Work Advanced", "W", "Scent Work"),
    ("SWAE", "Scent Work Advanced Elite", "W", "Scent Work"),
    ("SWD", "Scent Work Detective", "W", "Scent Work"),
    ("SWE", "Scent Work Excellent", "W", "Scent Work"),
    ("SWEE", "Scent Work Excellent Elite", "W", "Scent Work"),
    ("SWM", "Scent Work Master", "W", "Scent Work"),
    ("SWME", "Scent Work Master Elite", "W", "Scent Work"),
    ("SWN", "Scent Work Novice", "W", "Scent Work"),
    ("SWNE", "Scent Work Novice Elite", "W", "Scent Work"),
    # Time 2 Beat
    ("T2B", "Time 2 Beat", "U", "Time 2 Beat"),
    ("T2BP", "Time 2 Beat Preferred", "S", "Preferred Time 2 Beat"),
    # Tracking
    ("TD", "Tracking Dog", "T", "Tracking"),
    ("TDU", "Tracking Dog Urban", "T", "Tracking"),
    ("TDX", "Tracking Dog Excellent", "T", "Tracking"),
    # Therapy Dog
    ("THD", "Therapy Dog", "D", "Therapy Dog"),
    ("THDA", "Therapy Dog Advanced", "D", "Therapy Dog"),
    ("THDD", "Distinguished Therapy Dog", "D", "Therapy Dog"),
    ("THDN", "Therapy Dog Novice", "D", "Therapy Dog"),
    ("THDS", "Therapy Dog Supreme", "D", "Therapy Dog"),
    ("THDX", "Therapy Dog Excellent", "D", "Therapy Dog"),
    # Trick Dog
    ("TKA", "Advanced Trick Dog", "K", "Trick Dog"),
    ("TKE", "Trick Dog Elite Performer", "K", "Trick Dog"),
    ("TKI", "Intermediate Trick Dog", "K", "Trick Dog"),
    ("TKN", "Novice Trick Dog", "K", "Trick Dog"),
    ("TKP", "Trick Dog Performer", "K", "Trick Dog"),
    # Triple Q
    ("TQX", "Triple Q Excellent", "A", "Agility"),
    ("TQXP", "Triple Q Excellent Preferred", "P", "Preferred Agility Standard"),
    # Obedience Utility
    ("UD", "Utility Dog", "O", "Obedience"),
    ("UDX", "Utility Dog Excellent", "O", "Obedience"),
    # Versatile Companion Dog
    ("VCD1", "Versatile Companion Dog 1", "V", "Multi-sport (Obedience + Agility + Tracking)"),
    ("VCD2", "Versatile Companion Dog 2", "V", "Multi-sport (Obedience + Agility + Tracking)"),
    ("VCD3", "Versatile Companion Dog 3", "V", "Multi-sport (Obedience + Agility + Tracking)"),
    ("VCD4", "Versatile Companion Dog 4", "V", "Multi-sport (Obedience + Agility + Tracking)"),
    # Versatility
    ("VER", "Versatility", "O", "Obedience"),
    # Virtual Home Manners
    ("VHMA", "Virtual Home Manners Adult", "G", "Virtual Companion"),
    ("VHMP", "Virtual Home Manners Puppy", "G", "Virtual Companion"),
    # Variable Surface Tracking
    ("VST", "Variable Surface Tracking", "T", "Tracking"),
    # Virtual Scent Work
    ("VSWB", "Virtual Scent Work Beginner", "G", "Virtual Scent Work"),
    ("VSWE", "Virtual Scent Work Experienced", "G", "Virtual Scent Work"),
    ("VSWI", "Virtual Scent Work Intermediate", "G", "Virtual Scent Work"),
    # Agility XF
    ("XF", "Excellent FAST", "Y", "Agility FAST"),
    ("XFP", "Excellent FAST Preferred", "Z", "Preferred Agility FAST"),
    # Barn Hunt - AKC Other Recognized Titles section
    # Deborah confirmed (Q2, 2026-04-20) these actually appear in AKC catalog entries.
    # Source organization: Barn Hunt Association. AKC recognizes them for catalog display.
    ("RATN", "Novice Barn Hunt", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("RATO", "Open Barn Hunt", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("RATS", "Senior Barn Hunt", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("RATM", "Master Barn Hunt", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("RATCh", "Barn Hunt Champion", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("RATChX", "Barn Hunt Master Champion", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("CZ8B", "Crazy 8s Bronze", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("CZ8S", "Crazy 8s Silver", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("CZ8G", "Crazy 8s Gold", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
    ("CZ8P", "Crazy 8s Platinum", "BH", "Barn Hunt (AKC-recognized via Barn Hunt Association)"),
]

# Legacy compound titles from Deborah's catalog that AKC no longer publishes
# as discrete codes. Preserved because they appear in historical dog registered
# names and QTrial needs to render them correctly when parsing legacy data.
LEGACY_COMPOUND_SUFFIX_TITLES = [
    ("UDTD", "Utility Dog + Tracking Dog (legacy compound)", "V", "Multi-sport legacy"),
    ("UDTDX", "Utility Dog + Tracking Dog Excellent (legacy compound)", "V", "Multi-sport legacy"),
    ("UDXTDX", "Utility Dog Excellent + Tracking Dog Excellent (legacy compound)", "V", "Multi-sport legacy"),
    ("UDVST", "Utility Dog + Variable Surface Tracking (legacy compound)", "V", "Multi-sport legacy"),
    ("UDXVST", "Utility Dog Excellent + Variable Surface Tracking (legacy compound)", "V", "Multi-sport legacy"),
]


def write_akc_overrides_record():
    """Write an audit trail of AKC titles added beyond Deborah's source data.

    Every title in AKC_PREFIX_TITLES and AKC_SUFFIX_TITLES that is NOT present
    in Deborah's raw extract gets recorded here, so reviewers can see exactly
    what was added and why.
    """
    # Load codes present in Deborah's raw data
    deborah_prefix_codes = set()
    for r in read_access_csv(SRC / "raw_tblAKCTitlesPrefix.csv"):
        code = strip_trailing_whitespace(r.get("AKCPrefix", ""))
        if code:
            deborah_prefix_codes.add(code)

    deborah_suffix_codes = set()
    for r in read_access_csv(SRC / "raw_tblAKCTitlesSuffix.csv"):
        code = strip_trailing_whitespace(r.get("AKCSuffix", ""))
        if code:
            deborah_suffix_codes.add(code)

    rows = []
    for code, long_name, scope_code, scope_desc in AKC_PREFIX_TITLES:
        if code not in deborah_prefix_codes:
            rows.append({
                "category": "prefix_title",
                "code": code,
                "sport_scope_code": scope_code,
                "sport_scope_description": scope_desc,
                "long_name": long_name,
                "source_note": "Present in current AKC Titles & Abbreviations page; not present in Deborah's ObedienceData.accde (Lab Tested Databases catalog gap)",
            })

    for code, long_name, scope_code, scope_desc in AKC_SUFFIX_TITLES:
        if code not in deborah_suffix_codes:
            rows.append({
                "category": "suffix_title",
                "code": code,
                "sport_scope_code": scope_code,
                "sport_scope_description": scope_desc,
                "long_name": long_name,
                "source_note": "Present in current AKC Titles & Abbreviations page; not present in Deborah's ObedienceData.accde (Lab Tested Databases catalog gap)",
            })

    fieldnames = ["category", "code", "sport_scope_code", "sport_scope_description",
                  "long_name", "source_note"]
    write_csv(OUT / "akc_overrides_added.csv", fieldnames, rows)
    print(f"  akc_overrides_added.csv: {len(rows)} rows (AKC titles added beyond Deborah's catalog)")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("Cleaning AKC reference data extracts (v2 - includes new tables)...")
    print()
    print("--- Core reference data ---")
    clean_breed_groups()
    clean_breeds()
    clean_breed_varieties()
    clean_title_prefixes()
    clean_title_suffixes()
    clean_countries()
    clean_akc_country_mappings()
    clean_states()
    print()
    print("--- Class catalog and scoring ---")
    clean_canonical_classes()
    clean_om_points()
    clean_otch_points()
    clean_rally_rach_points()
    clean_obedience_exercises()
    clean_obedience_class_exercises()
    clean_jumps()
    print()
    print("--- AKC submission codes ---")
    clean_akc_xml_class_codes()
    clean_akc_xml_jump_heights()
    print()
    print("--- Non-AKC titles ---")
    clean_non_akc_title_suffixes()
    clean_non_akc_title_breed_restrictions()
    print()
    print("--- Operational defaults ---")
    clean_trial_time_calculations()
    print()
    print("--- AKC overrides (data added beyond Deborah's database) ---")
    write_akc_overrides_record()
    print()
    print("Done. Clean CSVs written to /home/claude/db_seed_akc_v2/")
