ALTER TABLE entry_line_results
    DROP CONSTRAINT IF EXISTS entry_line_results_rach_points_nonneg;

ALTER TABLE entry_line_results DROP COLUMN IF EXISTS rach_points;
ALTER TABLE entry_line_results DROP COLUMN IF EXISTS time_on_course;
ALTER TABLE entry_line_results DROP COLUMN IF EXISTS time_finished;
ALTER TABLE entry_line_results DROP COLUMN IF EXISTS time_started;
