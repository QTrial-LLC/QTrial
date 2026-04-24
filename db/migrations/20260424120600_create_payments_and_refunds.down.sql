DROP INDEX IF EXISTS refunds_payment_id_ix;
DROP INDEX IF EXISTS refunds_club_id_ix;
DROP TABLE IF EXISTS refunds;
DROP TYPE IF EXISTS refund_reason;

DROP INDEX IF EXISTS payments_club_deposited_ix;
DROP INDEX IF EXISTS payments_recorded_at_ix;
DROP INDEX IF EXISTS payments_entry_id_ix;
DROP INDEX IF EXISTS payments_club_id_ix;
DROP TABLE IF EXISTS payments;
-- payment_method ENUM is NOT dropped here: it was created by the
-- entries migration and is still referenced by the entries table.
