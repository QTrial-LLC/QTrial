-- Drop the junction first (it FKs the parent), then the parent.
-- Indexes and constraints drop with the tables.

DROP TABLE IF EXISTS combined_award_group_classes;
DROP TABLE IF EXISTS combined_award_groups;
