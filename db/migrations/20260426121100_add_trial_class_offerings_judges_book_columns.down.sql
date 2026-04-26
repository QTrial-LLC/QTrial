-- Drop the two judges-book columns. Any S3 object keys stored in
-- these columns are lost on rollback; the underlying S3 objects
-- themselves are not deleted (object-key columns are pointers,
-- not the artifacts).

ALTER TABLE trial_class_offerings
    DROP COLUMN signed_scan_pdf_object_key,
    DROP COLUMN pre_trial_blank_pdf_object_key;
