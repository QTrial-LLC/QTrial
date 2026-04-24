-- AKC submission tracking per trial.
--
-- For MVP Obedience and Rally submission, the package AKC accepts
-- is TWO electronic artifacts: the marked catalog PDF and the Form
-- JOVRY8 PDF (or the Obedience equivalent). Per Deborah's 2026-04-23
-- correction, the judges book that AKC receives is the PHYSICAL
-- original with the judge's wet signature, NOT a PDF. QTrial
-- generates judges book PDFs for pre-trial printing (tracked in a
-- separate artifact concept in a future PR); those printed copies
-- are signed by the judge and physically mailed to AKC. This table
-- tracks only the electronic submission: marked_catalog_object_key
-- and form_jovry8_object_key.
--
-- The earlier draft of DATA_MODEL.md §9 listed
-- judges_book_object_keys on this table; that column is
-- intentionally absent here and the doc is updated in the same PR.
--
-- xml_payload_object_key is nullable and reserved for post-MVP
-- Agility submission, which still uses AKC's XML schema. MVP
-- Obedience and Rally rows leave it NULL; submission_type
-- 'pdf_package' is the MVP value.
--
-- Two ENUMs are created in this migration. submission_type groups
-- submissions by the artifact shape. submission_status tracks the
-- lifecycle from local draft to AKC-accepted.

CREATE TYPE submission_type AS ENUM (
    'pdf_package',
    'xml',
    'csv'
);

CREATE TYPE submission_status AS ENUM (
    'draft',
    'generated',
    'submitted',
    'accepted',
    'rejected'
);

CREATE TABLE submission_records (
    id                              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Tenant root. ON DELETE CASCADE: hard delete of the club wipes
    -- its submission history.
    club_id                         UUID NOT NULL REFERENCES clubs(id) ON DELETE CASCADE,
    -- ON DELETE CASCADE on trial: a submission row is meaningless
    -- once its trial is gone. Soft delete via deleted_at does not
    -- cascade.
    trial_id                        UUID NOT NULL REFERENCES trials(id) ON DELETE CASCADE,
    submission_type                 submission_type NOT NULL,
    -- S3 object key for the marked catalog PDF. Populated for every
    -- MVP submission (submission_type = 'pdf_package').
    marked_catalog_object_key       TEXT,
    -- S3 object key for the Form JOVRY8 PDF (or the Obedience
    -- equivalent form). Populated for every MVP submission.
    form_jovry8_object_key          TEXT,
    -- Post-MVP Agility only; MVP rows leave NULL.
    xml_payload_object_key          TEXT,
    -- Defaults to 'rallyresults@akc.org' for Rally and the
    -- Obedience equivalent; per-event override permitted. Stored
    -- here rather than derived at send time so an audit of a
    -- historical submission is faithful to what was actually used.
    akc_destination_email           TEXT NOT NULL,
    -- AKC recording fee total computed at submission time
    -- (REQ-SUB-005). Pinned on the submission row so historical
    -- fee math survives future AKC rate changes.
    fee_total                       NUMERIC(10, 2) NOT NULL,
    submitted_at                    TIMESTAMPTZ,
    submitted_by_user_id            UUID REFERENCES users(id) ON DELETE SET NULL,
    status                          submission_status NOT NULL DEFAULT 'draft',
    -- Response payload when AKC eventually offers a structured
    -- response path; today this is NULL and the accept/reject
    -- signal is recorded by hand.
    akc_response                    JSONB,
    rejection_reason                TEXT,
    created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at                      TIMESTAMPTZ,
    created_by                      UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_by                      UUID REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT submission_records_fee_total_nonneg CHECK (
        fee_total >= 0
    )
);

CREATE INDEX submission_records_club_id_ix
    ON submission_records (club_id) WHERE deleted_at IS NULL;
CREATE INDEX submission_records_trial_id_ix
    ON submission_records (trial_id) WHERE deleted_at IS NULL;
-- Lifecycle queries ("show me every submission currently in
-- 'generated' status") filter by status and live rows.
CREATE INDEX submission_records_status_ix
    ON submission_records (status) WHERE deleted_at IS NULL;
