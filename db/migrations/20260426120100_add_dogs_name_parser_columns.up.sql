-- Add the four columns populated by the name parser (REQ-NAME-001).
--
-- The parser takes dogs.registered_name (free text, titles embedded)
-- and produces four outputs captured here:
--
--   * parsed_name_root          the registered name with every
--                                recognized title stripped,
--                                leaving just the dog's actual
--                                name. For example,
--                                "CH OTCH Rocky's Ruby Slippers
--                                 CDX GO RA CGC" yields
--                                "Rocky's Ruby Slippers".
--   * parsed_prefix_titles      recognized prefix title codes in
--                                order, e.g. {"CH","OTCH"}.
--   * parsed_suffix_titles      recognized suffix title codes in
--                                order, e.g. {"CDX","GO","RA","CGC"}.
--   * unparsed_title_tokens     title-like tokens that did not
--                                match the catalog (49 prefix + 244
--                                AKC core + 5 legacy + 10 Barn Hunt
--                                per the PR 2a seed scope).
--                                Preserved verbatim for
--                                trial-secretary review. Real data
--                                contains typos like UCGC (likely
--                                CGCU), WCCC? (literal question
--                                mark), and garbled concatenations
--                                like CGUWCX.
--
-- All four columns are nullable with no default. The parser is a
-- future Rust module (shared/src/name_parser or similar) that lands
-- in its own focused PR; adding the columns now avoids a second
-- schema migration when the parser ships. Rows inserted before the
-- parser runs carry NULL; the app layer tolerates NULL for unparsed
-- rows and can re-run the parser on demand.

ALTER TABLE dogs
    ADD COLUMN parsed_name_root TEXT,
    ADD COLUMN parsed_prefix_titles TEXT[],
    ADD COLUMN parsed_suffix_titles TEXT[],
    ADD COLUMN unparsed_title_tokens TEXT[];
