-- Reverse of 0002_add_transaction_indexes

DROP INDEX IF EXISTS idx_transaction_occurred_created;
DROP INDEX IF EXISTS idx_transaction_created_at;
DROP INDEX IF EXISTS idx_transaction_occurred_at;
DROP INDEX IF EXISTS idx_transaction_vendor_id;
DROP INDEX IF EXISTS idx_transaction_to_account_id;
DROP INDEX IF EXISTS idx_transaction_from_account_id;
DROP INDEX IF EXISTS idx_transaction_category_id;

ALTER TABLE transaction DROP COLUMN IF EXISTS created_at;
ALTER TABLE transaction ALTER COLUMN vendor_id SET NOT NULL;
